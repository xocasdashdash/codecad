#!/usr/bin/env bash
# Export one or more examples: run the script on a clean server-side
# document, save the DWG, then render the saved DWG to a 960x640 PNG.
#
# Prereqs (see AGENTS.md): server running on :8765 from the repo root
#   RUST_LOG=info target/release/cadview-server ./examples/
# and the repo root served for the renderer:
#   python3 -m http.server 8790 -d .
#
#   bash examples/lib/export.sh 01-flange-plate 02-fibonacci-spiral ...
set -euo pipefail
cd "$(dirname "$0")/../.."
API=${API:-http://localhost:8765/api/run}
AUTH="Authorization: Bearer cadview-local-dev"

run() {
  curl -sS -X POST "$API" -H "$AUTH" -H "Content-Type: application/json" -d "$1"
}

for name in "$@"; do
  name=${name%.js}
  echo "== $name"
  DOC="_work-$name"   # fresh per-example document (no stale blocks/linetypes)
  run "{\"doc_id\":\"$DOC\",\"program\":\"cad.clear()\"}" >/dev/null
  out=$(run "{\"doc_id\":\"$DOC\",\"exec\":\"examples/$name.js\"}")
  echo "$out" | python3 -c 'import json,sys; r=json.load(sys.stdin); print("   error:", r["error"]) if "error" in r else print("   value:", str(r.get("value"))[:300]); [print("   ", l) for l in (r.get("stdout") or "").splitlines()]; (r.get("stderr") or "").strip() and print("   stderr:", r["stderr"].strip()[:500])'
  if echo "$out" | grep -q '"error"'; then exit 1; fi
  run "{\"doc_id\":\"$DOC\",\"program\":\"return cad.saveDwg(\\\"examples/$name.dwg\\\")\"}" >/dev/null
  echo "   saved examples/$name.dwg"
done
node examples/lib/render.mjs "$@"
