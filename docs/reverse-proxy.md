# Reverse proxy support (work in progress)

This branch experiments with carrying the existing framed synchronization and RPC protocol over WebSockets on `/api/stream`. A normal HTTPS reverse proxy can forward the upgrade on the same origin as the app, so no public UDP port is needed. The old WebTransport listener is limited to loopback.

**Do not expose this branch yet.** The HTTP server injects its API token into the page; the token is therefore not user authentication. A reverse proxy must authenticate every request, including WebSocket upgrades and `/api/run`, before forwarding to the loopback server. It must preserve the original Host and replace client-supplied forwarded headers with trusted values. The WebSocket verifies Origin against Host and `X-Forwarded-Proto` and requires the page token as its first message. These measures do not replace proxy authentication.

Before deployment, test a full drawing load, server-to-browser updates, browser-to-server updates, RPC save, reconnect, unauthorized HTTP and WebSocket requests, cross-origin requests, and behavior after a restart. Verify the resulting DWG independently; overlay export has separate limitations.

A deployment may set `window.__CADVIEW_INITIAL_BOUNDS` in its frontend HTML to
`{min: [x0, y0], max: [x1, y1]}`. After the initial document sync, the browser
frames that area once without changing drawing entities; reconnects do not
reset the user's view.
