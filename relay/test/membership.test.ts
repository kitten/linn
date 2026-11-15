import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { canonical, evaluateAcceptance, parse, type Acceptance, type Membership } from "../src/membership.ts";

test("shared native fixtures have identical hashes, signatures and membership", async () => {
  const vectors = JSON.parse(await readFile(new URL("../../tests/fixtures/mesh-membership-v1.json", import.meta.url), "utf8")) as { accepted: Acceptance; state: Membership }[];
  const states = new Map<string, Membership>();
  for (const vector of vectors) {
    const parents = vector.accepted.proposal.event.heads.map((head) => states.get(head)!);
    const result = await evaluateAcceptance(vector.accepted, parents);
    assert.equal(canonical(result), canonical(vector.state));
    states.set(result.head, result);
    const tampered = structuredClone(vector.accepted);
    tampered.receipt.accepted++;
    await assert.rejects(evaluateAcceptance(tampered, parents));
  }
});

test("duplicate JSON properties and alternate encodings cannot enter signed messages", () => {
  for (const malformed of ['{"a":1,"a":1}', '{ "a":1}', '{"a":1.0}', '{"a":-0}']) {
    assert.throws(() => parse(malformed));
  }
});
