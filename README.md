# Homelab MCP starter

A minimal, working [Model Context Protocol](https://modelcontextprotocol.io) server
over Streamable HTTP. It ships with a few demo tools and one resource so you can
verify the whole path — client → Cloudflare Tunnel → this server — before you
write anything of your own.

## What's here

- `server.py` — the server: four demo tools (`ping`, `echo`, `add`, `server_info`),
  one resource, a `/health` route, and optional bearer-token auth.
- `requirements.txt`, `Dockerfile`, `docker-compose.yml` — containerised, with an
  optional Cloudflare Tunnel sidecar.
- `.env.example` — every config knob.

## 1. Run it locally

```bash
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
cp .env.example .env          # set MCP_AUTH_TOKEN to a random value
export $(grep -v '^#' .env | xargs)
python server.py
```

Verify in another terminal:

```bash
# Health (no auth required)
curl -s http://127.0.0.1:8000/health        # -> ok

# MCP handshake (auth required)
curl -s -X POST http://127.0.0.1:8000/mcp \
  -H "Authorization: Bearer $MCP_AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"curl","version":"0"}}}'
```

For an interactive view of the tools, point the MCP Inspector at it:

```bash
npx @modelcontextprotocol/inspector
# Transport: Streamable HTTP, URL: http://127.0.0.1:8000/mcp
# Add an Authorization header: Bearer <your token>
```

## 2. Run it in Docker

```bash
cp .env.example .env          # fill in MCP_AUTH_TOKEN (and TUNNEL_TOKEN later)
docker compose up -d --build mcp
```

## 3. Expose it with Cloudflare Tunnel

Both Claude and ChatGPT connect to your server *from their cloud*, so it needs a
public HTTPS URL. A tunnel gives you that without opening any inbound ports.

1. In the Cloudflare dashboard: **Zero Trust → Networks → Tunnels → Create a
   tunnel**. Choose the token install method and copy the tunnel token.
2. Put it in `.env` as `TUNNEL_TOKEN=...`.
3. In the tunnel's **Public Hostname** config, add e.g. `mcp.yourdomain.com` and
   set the service to `http://mcp:8000` (the compose service name).
4. Start the sidecar:

   ```bash
   docker compose up -d
   ```

Your endpoint is now `https://mcp.yourdomain.com/mcp`.

## 4. Connect it to Claude and ChatGPT

Same URL in both.

- **Claude** (Pro/Max/Team/Enterprise): Settings → Connectors → Add custom
  connector → paste `https://mcp.yourdomain.com/mcp`.
- **ChatGPT** (Plus/Pro/Business/Enterprise/Edu): Settings → Apps → Advanced
  settings → Developer mode → add the same URL.

## 5. A note on auth

The bearer token here is real protection and is perfect for curl, the MCP
Inspector, and any code you write. The catch: the consumer connector UIs don't
reliably let you set a custom `Authorization` header, so for Claude/ChatGPT
specifically you have two clean production options:

- **Cloudflare Access** in front of the public hostname (simplest for a homelab):
  gate the hostname with an Access policy. Good for connectors you drive yourself.
- **OAuth** — the path the connector "Connect" button is built for. FastMCP ships
  providers (GitHub, Google, Azure, Auth0, Keycloak, …) under
  `fastmcp.server.auth.providers`; wire one in to replace the bearer middleware.

Until you add one of those, keep `MCP_AUTH_TOKEN` set and treat the URL as a
secret. Never run with auth disabled on a public hostname.

## 6. Add your own tools

Replace the demo functions in `server.py`. The pattern is just a decorated
function — type hints become the input schema, and the docstring is what the
model reads to decide when to call it:

```python
@mcp.tool
def restart_service(name: str) -> str:
    """Restart a systemd service by name on the host. Returns the new status."""
    ...
```

Keep descriptions concrete about *when* to use the tool, validate every input
(these run with whatever privileges the container has), and prefer read-only
tools until you trust the setup.

## Security checklist

- [ ] `MCP_AUTH_TOKEN` set to a random 32-byte value, `.env` git-ignored.
- [ ] Public hostname gated by Cloudflare Access or OAuth before exposing write tools.
- [ ] Container runs least-privilege (no host Docker socket, scoped mounts only).
- [ ] Tools validate inputs and avoid shelling out with unsanitised arguments.