// Render example DWGs to 960x640 PNGs with headless Chromium (no app chrome).
//
//   python3 -m http.server 8790 -d .          # serve the repo root
//   node examples/lib/render.mjs 01-flange-plate [02-fibonacci-spiral ...]
//
// Each DWG is loaded through the embeddable <codecad-viewer> (render.html),
// so the PNG doubles as a DWG round-trip check. Needs dist-embed/ built
// (`just build-embed`). Env: CHROMIUM (default /usr/bin/chromium),
// BASE (default http://localhost:8790), RENDERER_FLAGS (extra flags).
import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const CHROMIUM = process.env.CHROMIUM || "/usr/bin/chromium";
const BASE = process.env.BASE || "http://localhost:8790";
const PORT = 9333 + Math.floor(Math.random() * 500);
const names = process.argv.slice(2);
if (!names.length) {
  console.error("usage: node render.mjs <example-name> ...");
  process.exit(1);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const chrome = spawn(
  CHROMIUM,
  [
    "--headless=new",
    `--remote-debugging-port=${PORT}`,
    `--user-data-dir=${mkdtempSync(join(tmpdir(), "codecad-render-"))}`,
    ...(process.env.WEBGPU === "0"
      ? ["--disable-features=WebGPU"]
      : ["--enable-unsafe-webgpu", "--enable-unsafe-swiftshader", "--enable-features=Vulkan", "--use-vulkan=swiftshader"]),
    "--use-angle=swiftshader",
    "--hide-scrollbars",
    "--window-size=960,640",
    ...(process.env.RENDERER_FLAGS ? process.env.RENDERER_FLAGS.split(" ") : []),
    "about:blank",
  ],
  { stdio: "ignore" },
);

async function cdpTarget() {
  for (let i = 0; i < 100; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
      const page = list.find((t) => t.type === "page");
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(100);
  }
  throw new Error("chromium did not start");
}

const ws = new WebSocket(await cdpTarget());
await new Promise((r) => ws.addEventListener("open", r, { once: true }));
let nextId = 1;
const pending = new Map();
const logs = [];
ws.addEventListener("message", (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  } else if (msg.method === "Runtime.consoleAPICalled") {
    logs.push(msg.params.args.map((a) => a.value ?? a.description).join(" "));
  } else if (msg.method === "Runtime.exceptionThrown") {
    logs.push(`EXCEPTION ${msg.params.exceptionDetails.exception?.description}`);
  }
});
const send = (method, params = {}) =>
  new Promise((r) => {
    const id = nextId++;
    pending.set(id, r);
    ws.send(JSON.stringify({ id, method, params }));
  });

await send("Runtime.enable");
await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", { width: 960, height: 640, deviceScaleFactor: 1, mobile: false });

let failed = false;
for (const name of names) {
  logs.length = 0;
  const file = name.endsWith(".dwg") ? name : `${name}.dwg`;
  await send("Page.navigate", { url: `${BASE}/examples/lib/render.html?f=${file}${process.env.ZBOUNDS ? `&z=${process.env.ZBOUNDS}` : ""}&t=${Date.now()}` });
  // Wait until the viewer has loaded the DWG (loading overlay hidden).
  let ok = false;
  for (let i = 0; i < 300; i++) {
    const r = await send("Runtime.evaluate", {
      expression: `(() => { const v = document.querySelector('codecad-viewer'); const l = v?.shadowRoot?.querySelector('.loading,.error'); return l ? (l.className + ':' + l.style.display + ':' + l.textContent) : 'none'; })()`,
      returnByValue: true,
    });
    const s = r.result?.result?.value || "";
    if (s.startsWith("error")) { logs.push(s); break; }
    if (s.startsWith("loading:none")) { ok = true; break; }
    await sleep(100);
  }
  // Vello on SwiftShader (CPU WebGPU) takes several seconds for the first
  // frame; until then the canvas composites as blank. Poll screenshots
  // until the image is non-trivial and stable.
  const snap = async () =>
    Buffer.from((await send("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: 960, height: 640, scale: 1 } })).result.data, "base64");
  if (process.env.DEBUG_EVAL) {
    const r = await send("Runtime.evaluate", { expression: process.env.DEBUG_EVAL, returnByValue: true, awaitPromise: true });
    console.log("DEBUG", JSON.stringify(r.result?.result?.value ?? r.result));
  }
  // A blank 960x640 frame is ~3 KB; wait for a bigger frame that stays
  // identical across three captures.
  let png = await snap();
  let stable = 0;
  for (let i = 0; i < 90 && stable < 2; i++) {
    await sleep(1000);
    const next = await snap();
    stable = next.length > 4000 && next.equals(png) ? stable + 1 : 0;
    png = next;
  }
  await sleep(Number(process.env.SETTLE_MS || 1000));
  png = await snap();
  if (process.env.ZOOM) {
    // Debug: ZOOM="x,y,w,h" writes a 3x close-up crop to ZOOM_OUT instead.
    const [x, y, width, height] = process.env.ZOOM.split(",").map(Number);
    const z = await send("Page.captureScreenshot", { format: "png", clip: { x, y, width, height, scale: 3 } });
    writeFileSync(process.env.ZOOM_OUT || "zoom.png", Buffer.from(z.result.data, "base64"));
    continue;
  }
  const out = process.env.ZBOUNDS ? (process.env.ZOOM_OUT || "zoom.png") : join(here, "..", `${name.replace(/\.dwg$/, "")}.png`);
  writeFileSync(out, png);
  console.log(`${ok ? "ok  " : "FAIL"} ${out}`);
  for (const l of logs) console.log(`     ${l}`);
  if (!ok) failed = true;
}
ws.close();
chrome.kill();
process.exit(failed ? 1 : 0);
