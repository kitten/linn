// @ref LLP 0008#protocol-resolutions
import canonicalize from "canonicalize";
import { sha256 } from "@noble/hashes/sha2.js";
import { hex, unhex } from "./protocol.ts";

export const MAX_EVENT_BYTES = 16 * 1024;
export const EVENT_DOMAIN = "linn mesh event v1\0";
export const RECEIPT_DOMAIN = "linn mesh receipt v1\0";
export const READ_DOMAIN = "linn mesh read v1\0";
const UNPAIRED_SURROGATE = /[\uD800-\uDFFF]/u;
export type Action =
  | { kind: "genesis"; actor: string; root: string; receipt_key: string }
  | { kind: "invite"; actor: string; key: string; receipt_key: string; not_before: number; expires: number }
  | { kind: "redeem"; actor: string; invitation: string }
  | { kind: "cancel"; actor: string; invitation: string }
  | { kind: "revoke"; actor: string; subject: string };
export interface Event { version: number; service: string; id: string; heads: string[]; action: Action }
export interface Proposal { event: Event; signatures: Record<string, string> }
export interface Receipt { version: number; service: string; event: string; accepted: number }
export interface Acceptance { proposal: Proposal; receipt: Receipt; signature: string }
export interface Invitation { issuer: string; key: string; not_before: number; expires: number; closed: boolean }
export interface Membership {
  head: string; service: string; receipt_key: string;
  roots: string[]; members: string[]; revoked: string[]; invitations: Record<string, Invitation>;
}

export class MembershipError extends Error {
  readonly status: number;
  constructor(message: string, status = 400) { super(message); this.status = status; }
}
function reject(message: string): never { throw new MembershipError(message); }
export function canonical(value: unknown): string {
  const result = canonicalize(value);
  if (result === undefined) return reject("invalid canonical value");
  return result;
}
export function message(domain: string, value: unknown): Uint8Array {
  return new TextEncoder().encode(domain + canonical(value));
}
export function hash(proposal: Proposal): string { return hex(sha256(new TextEncoder().encode(canonical(proposal)))); }
export function key(value: unknown): asserts value is string {
  if (typeof value !== "string") reject("invalid key/hash");
  try { unhex(value); } catch { reject("invalid key/hash"); }
}
function integer(value: unknown): asserts value is number {
  if (!Number.isSafeInteger(value) || (value as number) < 0) reject("invalid integer");
}
export function object(value: unknown, fields?: string[]): asserts value is Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) reject("expected object");
  if (fields && canonical(Object.keys(value).sort()) !== canonical(fields.sort())) reject("unknown or missing field");
}
export function origin(value: unknown): asserts value is string {
  if (typeof value !== "string" || value.length > 2048) reject("invalid service");
  let url: URL;
  try { url = new URL(value); } catch { return reject("invalid service origin"); }
  const local = ["localhost", "[::1]"].includes(url.hostname) || /^127(?:\.\d{1,3}){3}$/.test(url.hostname);
  if ((url.protocol !== "https:" && !(url.protocol === "http:" && local)) || url.origin !== value || url.port === "0") reject("invalid service origin");
}
export function parse(text: string): unknown {
  const value: unknown = JSON.parse(text, (key, value: unknown) => {
    // In Unicode mode, paired surrogates are one scalar outside this range.
    if (UNPAIRED_SURROGATE.test(key) || (typeof value === "string" && UNPAIRED_SURROGATE.test(value))) reject("invalid Unicode scalar");
    return value;
  });
  // Comparison rejects duplicate properties and alternate number spellings.
  if (canonical(value) !== text) reject("canonical JSON required");
  return value;
}
export async function verify(publicKey: string, signature: string, domain: string, value: unknown): Promise<void> {
  key(publicKey);
  let sig: Uint8Array;
  try { sig = unhex(signature, 64); } catch { return reject("invalid signature encoding"); }
  const imported = await crypto.subtle.importKey("raw", unhex(publicKey), "Ed25519", false, ["verify"]);
  if (!await crypto.subtle.verify("Ed25519", imported, sig, message(domain, value))) reject("invalid signature");
}
export async function sign(privateKey: CryptoKey, domain: string, value: unknown): Promise<string> {
  return hex(new Uint8Array(await crypto.subtle.sign("Ed25519", privateKey, message(domain, value))));
}
export async function validate(value: unknown): Promise<Proposal> {
  object(value, ["event", "signatures"]);
  object(value.event, ["version", "service", "id", "heads", "action"]);
  const event = value.event;
  if (event.version !== 1) reject("unsupported membership version");
  origin(event.service);
  key(event.id);
  if (!Array.isArray(event.heads) || event.heads.length > 2) reject("invalid heads");
  const heads = event.heads;
  for (const head of heads) key(head);
  if (heads.some((head, i) => i > 0 && heads[i - 1]! >= head)) reject("heads must be sorted and unique");
  object(event.action);
  key(event.action.actor);
  const action = event.action;
  switch (action.kind) {
    case "genesis": object(action, ["kind", "actor", "root", "receipt_key"]); key(action.root); key(action.receipt_key); break;
    case "invite":
      object(action, ["kind", "actor", "key", "receipt_key", "not_before", "expires"]);
      key(action.key); key(action.receipt_key); integer(action.not_before); integer(action.expires); break;
    case "redeem": case "cancel": object(action, ["kind", "actor", "invitation"]); key(action.invitation); break;
    case "revoke": object(action, ["kind", "actor", "subject"]); key(action.subject); break;
    default: reject("unknown membership action");
  }
  object(value.signatures);
  const signatures = Object.entries(value.signatures);
  if (signatures.length < 1 || signatures.length > 2 || new TextEncoder().encode(canonical(value)).length > MAX_EVENT_BYTES) reject("invalid proposal size");
  for (const [publicKey, signature] of signatures) {
    if (typeof signature !== "string") reject("invalid signature");
    await verify(publicKey, signature, EVENT_DOMAIN, event);
  }
  return value as unknown as Proposal;
}
const sorted = (values: Iterable<string>): string[] => [...new Set(values)].sort();

/** Signatures are checked by validate before entry into the storage transaction. */
export function evaluate(proposal: Proposal, parents: Membership[], accepted: number): Membership {
  integer(accepted);
  const { event, signatures } = proposal;
  const { action } = event;
  if (canonical(sorted(parents.map((p) => p.head))) !== canonical(event.heads) || parents.length !== event.heads.length) reject("missing or unexpected predecessor");
  const required = [action.actor];
  let state: Membership;
  if (action.kind === "genesis") {
    if (parents.length) reject("genesis has predecessors");
    state = { head: "", service: event.service, receipt_key: action.receipt_key, roots: [action.root], members: [action.actor], revoked: [], invitations: {} };
  } else {
    if (!parents.length || (action.kind !== "redeem" && parents.length !== 1)) reject("wrong predecessor count");
    state = structuredClone(parents[0]!);
    if (parents.some((p) => p.service !== event.service || p.receipt_key !== state.receipt_key)) reject("service or receipt key changed");
    if (parents.length === 2) {
      const second = parents[1]!;
      if (second.roots.some((root) => state.roots.includes(root))) reject("overlapping mesh histories");
      state.roots = sorted([...state.roots, ...second.roots]);
      state.revoked = sorted([...state.revoked, ...second.revoked]);
      state.members = sorted([...state.members, ...second.members]).filter((id) => !state.revoked.includes(id));
      for (const [id, invite] of Object.entries(second.invitations)) {
        if (state.invitations[id]) reject("duplicate invitation history");
        state.invitations[id] = structuredClone(invite);
      }
    }
    switch (action.kind) {
      case "redeem": {
        const source = parents.find((p) => p.invitations[action.invitation]);
        const invite = state.invitations[action.invitation];
        if (!source || !invite || invite.closed || accepted < invite.not_before || accepted >= invite.expires
          || !state.members.includes(invite.issuer) || state.revoked.includes(action.actor)) reject("expired, closed or unauthorized invitation");
        if (parents.length === 2) {
          if (source.members.includes(action.actor) || !parents.some((p) => p.head !== source.head && p.members.includes(action.actor))) reject("merge actor must belong to the other component");
        } else if (state.members.includes(action.actor)) reject("endpoint already enrolled");
        required.push(invite.key);
        invite.closed = true;
        state.members = sorted([...state.members, action.actor]);
        break;
      }
      case "invite":
        if (!state.members.includes(action.actor) || action.receipt_key !== state.receipt_key) reject("unauthorized invitation");
        if (action.expires - action.not_before !== 600 || accepted < action.not_before || accepted >= action.expires) reject("invalid invitation window");
        if (action.key === action.actor || state.members.includes(action.key) || state.revoked.includes(action.key)) reject("delegated key must be fresh");
        if (Object.values(state.invitations).some((invite) => invite.key === action.key)) reject("delegated key was already used");
        state.invitations[hash(proposal)] = { issuer: action.actor, key: action.key, not_before: action.not_before, expires: action.expires, closed: false };
        break;
      case "cancel": {
        if (!state.members.includes(action.actor)) reject("actor is not a member");
        const invite = state.invitations[action.invitation];
        if (!invite || invite.closed) reject("unknown or closed invitation");
        invite.closed = true;
        break;
      }
      case "revoke":
        if (!state.members.includes(action.actor) || !state.members.includes(action.subject)) reject("actor or subject is not a member");
        state.members = state.members.filter((member) => member !== action.subject);
        state.revoked = sorted([...state.revoked, action.subject]);
        break;
    }
  }
  if (canonical(sorted(required)) !== canonical(Object.keys(signatures).sort())) reject("wrong event signers");
  if (state.members.length > 256) reject("mesh exceeds member limit");
  state.head = hash(proposal);
  return state;
}

export async function acceptance(proposal: Proposal, accepted: number, receiptKey: CryptoKey): Promise<Acceptance> {
  const receipt = { version: 1, service: proposal.event.service, event: hash(proposal), accepted };
  return { proposal, receipt, signature: await sign(receiptKey, RECEIPT_DOMAIN, receipt) };
}
export async function evaluateAcceptance(value: unknown, parents: Membership[]): Promise<Membership> {
  object(value, ["proposal", "receipt", "signature"]);
  object(value.receipt, ["version", "service", "event", "accepted"]);
  const receipt = value.receipt;
  integer(receipt.accepted);
  const proposal = await validate(value.proposal);
  const state = evaluate(proposal, parents, receipt.accepted);
  if (receipt.version !== 1 || receipt.service !== state.service || receipt.event !== state.head || typeof value.signature !== "string") reject("receipt does not bind this event");
  await verify(state.receipt_key, value.signature, RECEIPT_DOMAIN, receipt);
  return state;
}
