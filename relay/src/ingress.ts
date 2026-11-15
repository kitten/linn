import { MAX_EVENT_BYTES } from "./membership.ts";

export interface IngressEnv {
  HTTP_LIMIT: RateLimit;
  CONNECT_LIMIT: RateLimit;
}

// Keep inexpensive probes and invalid requests outside the coordinator.
export async function ingress(request: Request, env: IngressEnv): Promise<Response | undefined> {
  const path = new URL(request.url).pathname;
  const method = ["/health", "/ping", "/v1/service", "/relay"].includes(path) ? "GET"
    : ["/v1/events", "/v1/history"].includes(path) ? "POST" : undefined;
  if (!method) return new Response("not found", { status: 404 });
  if (request.method !== method) return new Response("method not allowed", { status: 405, headers: { allow: method } });
  const length = request.headers.get("content-length");
  if (method === "POST" && length !== null) {
    if (!/^\d+$/.test(length)) return new Response("invalid content length", { status: 400 });
    if (Number(length) > MAX_EVENT_BYTES) return new Response("request too large", { status: 413 });
  }
  if (path === "/relay") {
    if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") return new Response("WebSocket required", { status: 426 });
    const versions = (request.headers.get("sec-websocket-protocol") ?? "").split(",").map((x) => x.trim());
    if (!versions.includes("iroh-relay-v2")) return new Response("iroh-relay-v2 required", { status: 400 });
  }
  // Cloudflare supplies this header. Never use client-controlled forwarding headers.
  const key = request.headers.get("cf-connecting-ip") ?? "local";
  try {
    if (!(await env.HTTP_LIMIT.limit({ key })).success
      || (path === "/relay" && !(await env.CONNECT_LIMIT.limit({ key })).success)) {
      return new Response("too many requests", { status: 429, headers: { "retry-after": "60", "cache-control": "no-store" } });
    }
  } catch {
    return new Response("rate limiter unavailable", { status: 503 });
  }
  if (path === "/health" || path === "/ping") {
    return new Response(path === "/health" ? "linn-relay: v1\n" : "", { headers: { "cache-control": "no-store" } });
  }
}
