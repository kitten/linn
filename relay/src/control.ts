import {
  acceptance, canonical, evaluate, hash, key, MAX_EVENT_BYTES, MembershipError,
  object, origin, parse, READ_DOMAIN, validate, verify,
  type Acceptance, type Membership,
} from "./membership.ts";
import { hex } from "./protocol.ts";
import { readState, writeState, deleteState } from "./state.ts";

interface ReceiptKey { private: number[]; public: string }
interface StoredEvent { acceptance: Acceptance; cursor: string }
interface OrderEntry { hash: string; root: string }
export interface ControlEnv { BOOTSTRAP_TOKEN?: string }

function json(value: unknown, status = 200): Response {
  return new Response(canonical(value), { status, headers: { "content-type": "application/json", "cache-control": "no-store" } });
}
function bootstrap(request: Request, token?: string): boolean {
  if (!token) return false;
  const encoder = new TextEncoder();
  const a = encoder.encode(request.headers.get("authorization") ?? "");
  const b = encoder.encode(`Bearer ${token}`);
  return a.length === b.length && crypto.subtle.timingSafeEqual(a, b);
}
export async function body(request: Request, deadlineMs = 10_000): Promise<unknown> {
  if (!request.body) throw new MembershipError("body required");
  const reader = request.body.getReader();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const expired = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new MembershipError("request body timed out", 408)), deadlineMs);
  });
  try {
    return await Promise.race([(async () => {
      const bytes = new Uint8Array(MAX_EVENT_BYTES);
      let length = 0;
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        if (length + value.length > MAX_EVENT_BYTES) throw new MembershipError("request too large", 413);
        bytes.set(value, length);
        length += value.length;
      }
      return parse(new TextDecoder("utf8", { fatal: true, ignoreBOM: true }).decode(bytes.subarray(0, length)));
    })(), expired]);
  } catch (error) {
    if (error instanceof MembershipError) throw error;
    throw new MembershipError("invalid control request body");
  } finally {
    if (timer !== undefined) clearTimeout(timer);
    void reader.cancel().catch(() => {});
  }
}

// @ref LLP 0008#pairing-and-mesh-membership
export class ControlPlane {
  private inFlight = 0;
  private migrated = false;
  private readonly ctx: DurableObjectState;
  private readonly env: ControlEnv;
  private readonly receipt: Promise<{ key: CryptoKey; id: string }>;
  constructor(ctx: DurableObjectState, env: ControlEnv) {
    this.ctx = ctx; this.env = env;
    this.receipt = ctx.blockConcurrencyWhile(async () => {
      this.migrated = await migrateDerivedIndexes(ctx.storage, 100);
      let stored = await ctx.storage.get<ReceiptKey>("receipt-key");
      if (!stored) {
        const pair = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]) as CryptoKeyPair;
        stored = {
          private: [...new Uint8Array(await crypto.subtle.exportKey("pkcs8", pair.privateKey) as ArrayBuffer)],
          public: hex(new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey) as ArrayBuffer)),
        };
        await ctx.storage.put("receipt-key", stored);
      }
      return { key: await crypto.subtle.importKey("pkcs8", new Uint8Array(stored.private), "Ed25519", false, ["sign"]), id: stored.public };
    });
  }

  async group(endpoint: string): Promise<string | undefined> {
    return this.ctx.storage.get<string>(`active:${endpoint}`);
  }

  async sameGroup(source: string, target: string): Promise<boolean> {
    const sourceKey = `active:${source}`;
    const targetKey = `active:${target}`;
    const groups = await this.ctx.storage.get<string>([sourceKey, targetKey]);
    const group = groups.get(sourceKey);
    return group !== undefined && group === groups.get(targetKey);
  }

  async fetch(request: Request): Promise<Response> {
    if (this.inFlight >= 4) return json({ error: "control service busy" }, 503);
    this.inFlight++;
    try {
      await this.receipt;
      if (!this.migrated) {
        this.migrated = await this.ctx.blockConcurrencyWhile(() => migrateDerivedIndexes(this.ctx.storage, 100));
        if (!this.migrated) return json({ error: "membership cache upgrade in progress; retry" }, 503);
      }
      const url = new URL(request.url);
      if (request.method === "GET" && url.pathname === "/v1/service") {
        return json({ version: 1, service: url.origin, receipt_key: (await this.receipt).id });
      }
      if (request.method !== "POST") return json({ error: "method not allowed" }, 405);
      if (url.pathname === "/v1/events") return await this.commit(request, url.origin);
      if (url.pathname === "/v1/history") return await this.history(request, url.origin);
      return json({ error: "not found" }, 404);
    } catch (error) {
      if (error instanceof MembershipError) return json({ error: error.message }, error.status);
      // Internal failures are uncertain outcomes, never definite rejections.
      // Exclude storage, signing, input and credential details from the response.
      return json({ error: "control service unavailable" }, 503);
    } finally { this.inFlight--; }
  }

  private async commit(request: Request, service: string): Promise<Response> {
    const proposal = await validate(await body(request));
    if (proposal.event.service !== service) throw new MembershipError("wrong service");
    const signer = await this.receipt;
    if (proposal.event.action.kind === "genesis") {
      if (proposal.event.action.receipt_key !== signer.id) throw new MembershipError("wrong receipt key");
    }
    const eventHash = hash(proposal);
    const accepted = await this.ctx.storage.transaction(async (tx) => {
      const existing = await tx.get<string>(`event-id:${proposal.event.id}`);
      if (existing) {
        if (existing !== eventHash) throw new MembershipError("event ID reused", 409);
        return (await tx.get<StoredEvent>(`event:${existing}`))!.acceptance;
      }
      if (proposal.event.action.kind === "genesis" && !bootstrap(request, this.env.BOOTSTRAP_TOKEN)) {
        throw new MembershipError("bootstrap authorization required", 401);
      }
      const parents: Membership[] = [];
      for (const head of proposal.event.heads) {
        const parent = await readState(tx, head);
        if (!parent) throw new MembershipError("unknown predecessor", 409);
        for (const root of parent.roots) {
          if (await tx.get<string>(`root:${root}`) !== head) throw new MembershipError("stale membership head", 409);
        }
        parents.push(parent);
      }
      const now = Math.floor(Date.now() / 1000);
      const state = evaluate(proposal, parents, now);
      if (state.receipt_key !== signer.id) throw new MembershipError("wrong receipt key");
      const action = proposal.event.action;
      if (action.kind === "genesis") {
        if (await tx.get(`root:${action.root}`) || await tx.get(`member:${action.actor}`)) throw new MembershipError("root or endpoint already enrolled", 409);
      }
      if (action.kind === "redeem" && parents.length === 1 && await tx.get(`member:${action.actor}`)) {
        throw new MembershipError("existing endpoint must merge both component heads", 409);
      }
      if (action.kind === "invite" && await tx.get(`invitation-key:${action.key}`)) throw new MembershipError("delegated key already registered", 409);
      const accepted = await acceptance(proposal, now, signer.key);
      const index = (await tx.get<number>("event-counter") ?? 0) + 1;
      if (!Number.isSafeInteger(index)) throw new MembershipError("history counter exhausted", 503);
      const cursor = String(index).padStart(16, "0");
      const root = state.roots[0]!;
      await tx.put(`event:${eventHash}`, { acceptance: accepted, cursor } satisfies StoredEvent);
      await tx.put(`event-id:${proposal.event.id}`, eventHash);
      await writeState(tx, state);
      for (const head of proposal.event.heads) await deleteState(tx, head);
      await tx.put(`order:${cursor}`, { hash: eventHash, root } satisfies OrderEntry);
      await tx.put(`component-order:${root}:${cursor}`, eventHash);
      await tx.put("event-counter", index);
      for (const root of state.roots) await tx.put(`root:${root}`, eventHash);
      for (const member of [...state.members, ...state.revoked]) await tx.put(`member:${member}`, root);
      for (const member of state.members) await tx.put(`active:${member}`, root);
      for (const member of state.revoked) await tx.delete(`active:${member}`);
      if (action.kind === "invite") await tx.put(`invitation-key:${action.key}`, { root, invitation: eventHash });
      return accepted;
    });
    return json(accepted);
  }

  private async history(request: Request, service: string): Promise<Response> {
    const value = await body(request);
    object(value, ["request", "signature"]);
    object(value.request, ["version", "service", "key", "expires", "nonce", "after", "invitation"]);
    const auth = value.request;
    origin(auth.service); key(auth.key); key(auth.nonce);
    const now = Math.floor(Date.now() / 1000);
    if (auth.version !== 1 || auth.service !== service || !Number.isSafeInteger(auth.expires)
      || (auth.expires as number) <= now || (auth.expires as number) > now + 120
      || !(auth.after === null || (typeof auth.after === "string" && /^\d{16}$/.test(auth.after)))
      || !(auth.invitation === null || typeof auth.invitation === "string") || typeof value.signature !== "string") {
      throw new MembershipError("invalid read authorization", 401);
    }
    if (auth.invitation !== null) key(auth.invitation);
    await verify(auth.key, value.signature, READ_DOMAIN, auth);
    const publicKey = auth.key;
    // A pinned read transaction prevents concurrent revocation leaking a later page.
    const page = await this.ctx.storage.transaction(async (tx) => {
      const now = Math.floor(Date.now() / 1000);
      if ((auth.expires as number) <= now) throw new MembershipError("read authorization expired", 401);
      const delegation = auth.invitation === null ? undefined : await tx.get<{ root: string; invitation: string }>(`invitation-key:${auth.key}`);
      const root = auth.invitation === null ? await tx.get<string>(`member:${auth.key}`) : delegation?.root;
      const head = root ? await tx.get<string>(`root:${root}`) : undefined;
      const state = head ? await readState(tx, head) : undefined;
      if (!state) throw new MembershipError("not enrolled", 403);
      if (auth.invitation === null) {
        if (!state.members.includes(publicKey)) throw new MembershipError("not authorized", 403);
      } else {
        const invitation = state.invitations[auth.invitation as string];
        if (delegation?.invitation !== auth.invitation || !invitation || invitation.key !== auth.key
          || invitation.closed || now < invitation.not_before || now >= invitation.expires
          || !state.members.includes(invitation.issuer)) throw new MembershipError("invitation unavailable", 403);
      }
      const knownHead = request.headers.get("x-linn-known-head");
      if (knownHead !== null) key(knownHead);
      if (knownHead === state.head) {
        return { version: 1, head: state.head, events: [], after: auth.after, more: false };
      }
      // Merge bounded ranges from this component's root ancestries only.
      const candidates: { cursor: string; hash: string }[] = [];
      for (const root of state.roots) {
        const prefix = `component-order:${root}:`;
        const entries = await tx.list<string>({ prefix, startAfter: auth.after === null ? undefined : `${prefix}${auth.after}`, limit: 16 });
        for (const [key, hash] of entries) candidates.push({ cursor: key.slice(prefix.length), hash });
        candidates.sort((a, b) => a.cursor.localeCompare(b.cursor));
        candidates.splice(16);
      }
      const selected = candidates;
      const events: Acceptance[] = [];
      for (const row of selected) events.push((await tx.get<StoredEvent>(`event:${row.hash}`))!.acceptance);
      const last = selected.at(-1)?.cursor ?? auth.after;
      return { version: 1, head: state.head, events, after: last, more: selected.length === 16 };
    });
    return json(page);
  }
}

export async function migrateDerivedIndexes(storage: DurableObjectStorage, budgetMs = Infinity): Promise<boolean> {
  const started = performance.now();
  do {
    const version = await storage.get<number>("derived-index-version");
    if (version === 2) return true;
    if (version !== undefined && (!Number.isSafeInteger(version) || version < 0 || version > 2)) {
      throw new MembershipError("unsupported membership cache version", 503);
    }
    await storage.transaction(async (tx) => {
      const cursor = await tx.get<string>("derived-index-cursor");
      const entries = await tx.list<OrderEntry>({ prefix: "order:", startAfter: cursor, limit: 1 });
      for (const [key, row] of entries) {
        await tx.put(`component-order:${row.root}:${key.slice("order:".length)}`, row.hash);
        if (await tx.get<string>(`root:${row.root}`) === row.hash) {
          const state = await readState(tx, row.hash);
          if (state) await writeState(tx, state);
        } else {
          await deleteState(tx, row.hash);
        }
      }
      if (entries.size === 0) {
        await tx.put("derived-index-version", 2);
        await tx.delete("derived-index-cursor");
      } else {
        await tx.put("derived-index-cursor", [...entries.keys()].at(-1)!);
      }
    });
  } while (performance.now() - started < budgetMs);
  return await storage.get<number>("derived-index-version") === 2;
}
