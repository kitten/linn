import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import type { Miniflare } from "miniflare";
import { createRuntime } from "./runtime.ts";
import { canonical, EVENT_DOMAIN, evaluateAcceptance, hash, READ_DOMAIN, sign, type Acceptance, type Action, type Event, type Membership, type Proposal } from "../src/membership.ts";
import { hex } from "../src/protocol.ts";

let runtime: Miniflare;
let receiptKey: string;
const service = "https://local";
const token = "private-local-control-test";
before(async () => {
  runtime = await createRuntime(token);
  receiptKey = (await (await runtime.dispatchFetch(`${service}/v1/service`)).json() as { receipt_key: string }).receipt_key;
});
after(async () => { await runtime?.dispose(); });
const id = () => hex(crypto.getRandomValues(new Uint8Array(32)));
async function identity() {
  const pair = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
  return { privateKey: pair.privateKey, id: hex(new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey))) };
}
type Identity = Awaited<ReturnType<typeof identity>>;
async function proposal(heads: string[], action: Action, keys: Identity[]): Promise<Proposal> {
  const event: Event = { version: 1, service, id: id(), heads: heads.sort(), action };
  return { event, signatures: Object.fromEntries(await Promise.all(keys.map(async (key) => [key.id, await sign(key.privateKey, EVENT_DOMAIN, event)]))) };
}
async function post(value: Proposal, credential = token) {
  return runtime.dispatchFetch(`${service}/v1/events`, { method: "POST", headers: { authorization: `Bearer ${credential}` }, body: canonical(value) });
}
async function commit(value: Proposal, parents: Membership[]): Promise<{ accepted: Acceptance; state: Membership }> {
  const response = await post(value);
  assert.equal(response.status, 200, await response.clone().text());
  const accepted = await response.json() as Acceptance;
  return { accepted, state: await evaluateAcceptance(accepted, parents) };
}
async function genesis(owner: Identity) {
  return commit(await proposal([], { kind: "genesis", actor: owner.id, root: id(), receipt_key: receiptKey }, [owner]), []);
}
async function invitation(owner: Identity, state: Membership, delegated: Identity) {
  const now = Math.floor(Date.now() / 1000);
  return commit(await proposal([state.head], { kind: "invite", actor: owner.id, key: delegated.id, receipt_key: receiptKey, not_before: now, expires: now + 600 }, [owner]), [state]);
}
async function history(owner: Identity, invitation: string | null = null, after: string | null = null, knownHead?: string) {
  const request = { version: 1, service, key: owner.id, expires: Math.floor(Date.now() / 1000) + 60, nonce: id(), after, invitation };
  return runtime.dispatchFetch(`${service}/v1/history`, { method: "POST", headers: knownHead ? { "x-linn-known-head": knownHead } : {}, body: canonical({ request, signature: await sign(owner.privateKey, READ_DOMAIN, request) }) });
}

test("bootstrap, signature proofs, private history, atomic admission and exact retries", async () => {
  const a = await identity(); const b = await identity(); const delegated = await identity();
  const initial = await proposal([], { kind: "genesis", actor: a.id, root: id(), receipt_key: receiptKey }, [a]);
  assert.equal((await post(initial, "wrong")).status, 401);
  const first = await commit(initial, []);
  assert.equal(canonical(await (await post(initial, "wrong")).json()), canonical(first.accepted));
  const invite = await invitation(a, first.state, delegated);
  assert.equal((await history(b)).status, 403);
  const pending = await history(delegated, invite.state.head);
  assert.equal(pending.status, 200);
  const join = await proposal([invite.state.head], { kind: "redeem", actor: b.id, invitation: invite.state.head }, [b, delegated]);
  const admitted = await commit(join, [invite.state]);
  assert.equal(admitted.state.members.length, 2);
  assert.equal(canonical(await (await post(join)).json()), canonical(admitted.accepted));
  assert.equal((await history(delegated, invite.state.head)).status, 403);
  const reused = await proposal([admitted.state.head], { kind: "redeem", actor: (await identity()).id, invitation: invite.state.head }, [b, delegated]);
  assert.equal((await post(reused)).status, 400);
  // Revocation does not change the historical result returned for a lost success reply.
  const revoked = await commit(await proposal([admitted.state.head], { kind: "revoke", actor: a.id, subject: b.id }, [a]), [admitted.state]);
  assert.ok(revoked.state.revoked.includes(b.id));
  assert.equal((await history(b)).status, 403);
  assert.equal(canonical(await (await post(join)).json()), canonical(admitted.accepted));
  assert.equal((await history(a)).status, 200);
  const tampered = structuredClone(initial);
  tampered.event.id = id();
  assert.equal((await post(tampered)).status, 400);
});

test("simultaneous membership changes have one winner and stale proposals cannot mutate", async () => {
  const a = await identity(); const b = await identity(); const delegated = await identity();
  const first = await genesis(a);
  const invite = await invitation(a, first.state, delegated);
  const joined = await commit(await proposal([invite.state.head], { kind: "redeem", actor: b.id, invitation: invite.state.head }, [b, delegated]), [invite.state]);
  const revokeA = await proposal([joined.state.head], { kind: "revoke", actor: b.id, subject: a.id }, [b]);
  const revokeB = await proposal([joined.state.head], { kind: "revoke", actor: a.id, subject: b.id }, [a]);
  const results = await Promise.all([post(revokeA), post(revokeB)]);
  assert.deepEqual(results.map((result) => result.status).sort(), [200, 409]);
  const accepted = await results.find((result) => result.status === 200)!.json() as Acceptance;
  const final = await evaluateAcceptance(accepted, [joined.state]);
  assert.equal(final.members.length, 1);
  const loser = accepted.proposal.event.action.actor === a.id ? b : a;
  const winner = loser.id === a.id ? b : a;
  const retried = await proposal([final.head], { kind: "revoke", actor: loser.id, subject: winner.id }, [loser]);
  assert.equal((await post(retried)).status, 400);
});

test("merges atomically replace both heads and survive object eviction", async () => {
  const a = await identity(); const b = await identity(); const delegated = await identity();
  const first = await genesis(a); const second = await genesis(b);
  const invite = await invitation(a, first.state, delegated);
  const merge = await proposal([invite.state.head, second.state.head], { kind: "redeem", actor: b.id, invitation: hash(invite.accepted.proposal) }, [b, delegated]);
  const merged = await commit(merge, [invite.state, second.state]);
  assert.equal(merged.state.roots.length, 2);
  await runtime.unsafeEvictDurableObject("relay", "RelayCoordinator", { name: "private-mesh-v1", webSockets: "hibernate" });
  assert.equal((await (await runtime.dispatchFetch(`${service}/v1/service`)).json() as { receipt_key: string }).receipt_key, receiptKey);
  assert.equal((await post(await proposal([second.state.head], { kind: "revoke", actor: b.id, subject: b.id }, [b]))).status, 409);
  const read = await history(b);
  assert.equal(read.status, 200);
  assert.equal((await read.json() as { head: string }).head, merged.state.head);
});

test("noncanonical and oversized requests fail without storing a proposal", async () => {
  assert.equal((await runtime.dispatchFetch(`${service}/v1/events`, { method: "POST", body: '{"x":1,"x":2}' })).status, 400);
  assert.equal((await runtime.dispatchFetch(`${service}/v1/events`, { method: "POST", body: " ".repeat(17000) })).status, 413);
});

test("history skips unrelated components and pages merged root ancestry in event order", async () => {
  const a = await identity(); const b = await identity(); const c = await identity();
  const first = await genesis(a);
  let other = await genesis(b);
  const expected = [first.state.head, other.state.head];
  for (let i = 0; i < 35; i++) {
    other = await invitation(b, other.state, await identity());
    expected.push(other.state.head);
  }
  const invite = await invitation(a, first.state, await identity());
  // A tiny component returns its own two events immediately, despite global traffic.
  const tiny = await (await history(a)).json() as { events: Acceptance[]; more: boolean };
  assert.deepEqual(tiny.events.map((e) => hash(e.proposal)), [first.state.head, invite.state.head]);
  assert.equal(tiny.more, false);
  // Make a fresh delegation whose private key is available for the merge.
  const delegation = await identity();
  const joining = await invitation(a, invite.state, delegation);
  const unrelated = await genesis(c);
  const merged = await commit(await proposal([joining.state.head, other.state.head],
    { kind: "redeem", actor: b.id, invitation: joining.state.head }, [b, delegation]), [joining.state, other.state]);
  expected.push(invite.state.head, joining.state.head, merged.state.head);
  let after: string | null = null;
  const received: string[] = [];
  for (let page = 0; page < 4; page++) {
    const result = await (await history(b, null, after)).json() as { events: Acceptance[]; more: boolean; after: string };
    assert.ok(result.events.length > 0);
    received.push(...result.events.map((e) => hash(e.proposal)));
    after = result.after;
    if (!result.more) break;
  }
  assert.deepEqual(received, expected);
  assert.ok(!received.includes(unrelated.state.head));
});


test("matching history heads omit replay only after current reader authorization", async () => {
  const owner = await identity();
  const stranger = await identity();
  const first = await genesis(owner);
  const unchanged = await history(owner, null, null, first.state.head);
  assert.equal(unchanged.status, 200);
  assert.deepEqual(await unchanged.json(), { version: 1, head: first.state.head, events: [], after: null, more: false });
  assert.equal((await history(stranger, null, null, first.state.head)).status, 403);
  const invited = await invitation(owner, first.state, await identity());
  const changed = await history(owner, null, null, first.state.head);
  assert.equal(changed.status, 200);
  const page = await changed.json() as { events: Acceptance[]; head: string };
  assert.equal(page.head, invited.state.head);
  assert.equal(page.events.length, 2);
  const revoked = await commit(await proposal([invited.state.head], { kind: "revoke", actor: owner.id, subject: owner.id }, [owner]), [invited.state]);
  assert.equal((await history(owner, null, null, revoked.state.head)).status, 403);
});
