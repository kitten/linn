// @ref LLP 0008#derived-membership-storage-and-history
// Derived current-head caches. Immutable signed events remain the source of truth.
import { canonical, MembershipError, type Membership } from "./membership.ts";

const CHUNK = 64 * 1024;
type Stored = Membership | { chunks: number };
function chunkCount(stored: { chunks: number }): number {
  if (!Number.isSafeInteger(stored.chunks) || stored.chunks < 1) throw new MembershipError("invalid state cache", 503);
  return stored.chunks;
}
export async function readState(tx: DurableObjectTransaction, head: string): Promise<Membership | undefined> {
  const stored = await tx.get<Stored>(`state:${head}`);
  if (!stored) return;
  if ("head" in stored) return stored; // Previous deployment's aggregate cache.
  chunkCount(stored);
  const parts: string[] = [];
  for (let start = 0; start < stored.chunks; start += 64) {
    const keys = Array.from({ length: Math.min(64, stored.chunks - start) }, (_, i) => `state:${head}:${start + i}`);
    const values = await tx.get<string>(keys);
    for (const key of keys) {
      const value = values.get(key);
      if (value === undefined) throw new MembershipError("incomplete state cache", 503);
      parts.push(value);
    }
  }
  return JSON.parse(parts.join("")) as Membership;
}
export async function writeState(tx: DurableObjectTransaction, state: Membership): Promise<void> {
  const text = canonical(state);
  let chunks = 0;
  for (let offset = 0; offset < text.length; offset += CHUNK) {
    await tx.put(`state:${state.head}:${chunks++}`, text.slice(offset, offset + CHUNK));
  }
  await tx.put(`state:${state.head}`, { chunks });
}
export async function deleteState(tx: DurableObjectTransaction, head: string): Promise<void> {
  const stored = await tx.get<Stored>(`state:${head}`);
  if (stored && !("head" in stored)) {
    chunkCount(stored);
    for (;;) {
      const keys = [...(await tx.list({ prefix: `state:${head}:`, limit: 64 })).keys()];
      if (keys.length) await tx.delete(keys);
      if (keys.length < 64) break;
    }
  }
  await tx.delete(`state:${head}`);
}
