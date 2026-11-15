// Persist these token buckets in each hibernatable socket attachment.
export interface MessageBudget {
  messages?: number;
  controls?: number;
  messagesAt?: number;
}

export function admitMessage(state: MessageBudget, now: number, control: boolean): boolean {
  const elapsed = Math.max(0, now - (state.messagesAt ?? now)) / 1000;
  state.messages = Math.min(16384, (state.messages ?? 16384) + elapsed * 8192);
  state.controls = Math.min(64, (state.controls ?? 64) + elapsed * 32);
  state.messagesAt = Math.max(now, state.messagesAt ?? now);
  if (state.messages < 1 || (control && state.controls < 1)) return false;
  state.messages--;
  if (control) state.controls--;
  return true;
}

export const PING_MS = 15_000;
export interface Heartbeat {
  awaitingPong?: number;
  pingAt?: number;
}

export function heartbeatDeadline(state: Heartbeat, now: number): number {
  return state.awaitingPong !== undefined ? state.awaitingPong + 2 * PING_MS : state.pingAt ?? now;
}
