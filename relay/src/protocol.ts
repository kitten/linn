// Wire adapter for iroh-relay 1.4.1. See THIRD_PARTY_NOTICES.md.
import { blake3 } from "@noble/hashes/blake3.js";

export const MAX_PACKET = 64 * 1024;
const CONTEXT = new TextEncoder().encode("iroh-relay handshake v1 challenge signature");

export function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

export function unhex(value: string): Uint8Array {
  if (!/^[0-9a-f]{64}$/.test(value)) throw new Error("invalid endpoint key");
  return Uint8Array.from(value.match(/../g)!, (byte) => Number.parseInt(byte, 16));
}

export function frame(kind: number, payload: Uint8Array = new Uint8Array()): Uint8Array {
  if (!Number.isInteger(kind) || kind < 0 || kind > 63) throw new Error("invalid tag");
  const result = new Uint8Array(payload.length + 1);
  result[0] = kind;
  result.set(payload, 1);
  return result;
}

export function decode(input: ArrayBuffer): { kind: number; payload: Uint8Array } {
  const data = new Uint8Array(input);
  if (!data.length || data.length > MAX_PACKET + 8) throw new Error("invalid frame length");
  const width = 1 << (data[0]! >> 6);
  if (data.length < width) throw new Error("truncated tag");
  let kind = data[0]! & 63;
  for (let i = 1; i < width; i++) kind = kind * 256 + data[i]!;
  if (kind > 13 || !Number.isSafeInteger(kind)) throw new Error("unknown frame tag");
  const payload = data.subarray(width);
  if (payload.length > MAX_PACKET) throw new Error("oversized frame");
  return { kind, payload };
}

export function challengeMessage(challenge: Uint8Array): Uint8Array {
  if (challenge.length !== 16) throw new Error("invalid challenge");
  return blake3(challenge, { context: CONTEXT });
}

export async function authenticate(input: ArrayBuffer, challenge: Uint8Array): Promise<string> {
  const { kind, payload } = decode(input);
  // Postcard: a 32-byte public-key array, then serde_bytes length 64 + signature.
  if (kind !== 1 || payload.length !== 97 || payload[32] !== 64) {
    throw new Error("invalid client authentication");
  }
  const publicKey = payload.slice(0, 32);
  const key = await crypto.subtle.importKey("raw", publicKey, "Ed25519", false, ["verify"]);
  if (!await crypto.subtle.verify("Ed25519", key, payload.slice(33), challengeMessage(challenge))) {
    throw new Error("invalid authentication signature");
  }
  return hex(publicKey);
}

export type ClientMessage =
  | { type: "ping"; payload: Uint8Array }
  | { type: "pong"; payload: Uint8Array }
  | { type: "datagram"; destination: string; batch: boolean; payload: Uint8Array };

export function clientMessage(input: ArrayBuffer): ClientMessage {
  const { kind, payload } = decode(input);
  if (kind === 9 || kind === 10) {
    if (payload.length !== 8) throw new Error("invalid ping/pong");
    return { type: kind === 9 ? "ping" : "pong", payload };
  }
  if (kind !== 4 && kind !== 5) throw new Error("unexpected client message");
  const header = kind === 5 ? 35 : 33;
  if (payload.length < header) throw new Error("truncated datagram");
  // Only two ECN bits are defined. A zero batch size is not a real batch.
  if (payload[32]! > 3 || (kind === 5 && payload[33] === 0 && payload[34] === 0)) {
    throw new Error("invalid datagram metadata");
  }
  return {
    type: "datagram", destination: hex(payload.subarray(0, 32)), batch: kind === 5,
    payload: payload.subarray(32),
  };
}

export function forward(source: string, message: Extract<ClientMessage, { type: "datagram" }>): Uint8Array {
  const payload = new Uint8Array(32 + message.payload.length);
  payload.set(unhex(source));
  payload.set(message.payload, 32);
  return frame(message.batch ? 7 : 6, payload);
}
