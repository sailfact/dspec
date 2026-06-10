"""
Homelab MCP starter server.

A minimal, working Model Context Protocol server over Streamable HTTP.
Exposes a few demo tools and one resource so you can verify the full path
(client -> Cloudflare Tunnel -> this server) end to end, then replace the
demos with your own tools.

Run locally:    python server.py
Run in Docker:  see Dockerfile / docker-compose.yml
"""

import os
import socket
import datetime as dt

from fastmcp import FastMCP
from starlette.middleware import Middleware
from starlette.responses import JSONResponse, PlainTextResponse
from starlette.types import ASGIApp, Receive, Scope, Send

# --- Configuration (all via environment) ---------------------------------
HOST = os.environ.get("MCP_HOST", "0.0.0.0")
PORT = int(os.environ.get("MCP_PORT", "8000"))
MCP_PATH = os.environ.get("MCP_PATH", "/mcp")
# If set, every request to MCP_PATH must send: Authorization: Bearer <token>
AUTH_TOKEN = os.environ.get("MCP_AUTH_TOKEN", "").strip()

mcp = FastMCP("homelab-starter")


# --- Demo tools -----------------------------------------------------------
# Each @mcp.tool function becomes a tool the model can call. The docstring
# is what the model reads to decide when to use it, and type hints become
# the input schema automatically. Replace these with your own.

@mcp.tool
def ping() -> str:
    """Liveness check. Returns 'pong' if the server is reachable."""
    return "pong"


@mcp.tool
def echo(message: str) -> str:
    """Echo a message straight back. Useful for confirming round-trips."""
    return message


@mcp.tool
def add(a: float, b: float) -> float:
    """Add two numbers and return the sum."""
    return a + b


@mcp.tool
def server_info() -> dict:
    """Report basic info about the host this server runs on."""
    return {
        "hostname": socket.gethostname(),
        "utc_time": dt.datetime.now(dt.timezone.utc).isoformat(),
        "server": "homelab-starter",
    }


# --- Demo resource --------------------------------------------------------
# Resources are read-only data the client can pull into context on request.
@mcp.resource("info://about")
def about() -> str:
    """A short description of what this server is."""
    return (
        "Homelab MCP starter. Swap the demo tools in server.py for your own, "
        "then expose this over HTTPS (e.g. via Cloudflare Tunnel)."
    )


# --- Optional bearer-token auth ------------------------------------------
# Lightweight, framework-agnostic gate. Exempts the /health endpoint so
# uptime checks and the tunnel don't need the token. This is defense in
# depth; for the consumer apps, prefer OAuth or Cloudflare Access (see README).
class BearerAuthMiddleware:
    def __init__(self, app: ASGIApp, token: str) -> None:
        self.app = app
        self.expected = f"Bearer {token}"

    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return
        path = scope.get("path", "")
        if path == "/health":
            await self.app(scope, receive, send)
            return
        headers = dict(scope.get("headers") or [])
        provided = headers.get(b"authorization", b"").decode()
        if provided != self.expected:
            await JSONResponse({"error": "unauthorized"}, status_code=401)(
                scope, receive, send
            )
            return
        await self.app(scope, receive, send)


@mcp.custom_route("/health", methods=["GET"])
async def health(_request):
    return PlainTextResponse("ok")


def build_app():
    middleware = []
    if AUTH_TOKEN:
        middleware.append(Middleware(BearerAuthMiddleware, token=AUTH_TOKEN))
    return mcp.http_app(path=MCP_PATH, middleware=middleware)


app = build_app()


if __name__ == "__main__":
    import uvicorn

    if not AUTH_TOKEN:
        print("WARNING: MCP_AUTH_TOKEN is not set - the server is unauthenticated.")
    uvicorn.run(app, host=HOST, port=PORT)