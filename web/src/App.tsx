import { type DragEvent, useCallback, useEffect, useState } from "react";
import { cad, type EntityJson, getInitialSessionId, onTabRenamed } from "./cad";
import { DocumentPicker } from "./DocumentPicker";
import { exampleDwgUrl } from "./examples-manifest";
import { LayerPanel } from "./LayerPanel";
import { TabBar } from "./TabBar";
import { ViewportContainer } from "./ViewportContainer";

interface Tab {
  id: string;
  label: string;
}

interface LayerState {
  name: string;
  color: [number, number, number];
  visible: boolean;
  entityCount: number;
}

let _newDrawingCounter = 0;

export function App() {
  const initId = getInitialSessionId();
  const initLabel =
    initId
      .replace(/\.dwg$/i, "")
      .split("/")
      .pop() || initId;
  const [tabs, setTabs] = useState<Tab[]>([{ id: initId, label: initLabel }]);
  const [activeTabId, setActiveTabId] = useState<string>(initId);
  const [comparePrimaryId, setComparePrimaryId] = useState<string>(initId);
  const [compareTabId, setCompareTabId] = useState<string | null>(null);
  const [layers, setLayers] = useState<LayerState[]>([]);
  const [panelOpen, setPanelOpen] = useState(false);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [markupMode, setMarkupMode] = useState(false);

  // Register tab rename callback (called by cad.save/saveDwg)
  // biome-ignore lint/correctness/useExhaustiveDependencies: runs once on mount
  useEffect(() => {
    onTabRenamed((sessionId, newLabel) => {
      setTabs((prev) =>
        prev.map((t) => (t.id === sessionId ? { ...t, label: newLabel } : t)),
      );
    });

    // Auto-load DWG from ?file=URL parameter
    const fileUrl = new URLSearchParams(window.location.search).get("file");
    if (fileUrl) {
      fetch(fileUrl)
        .then((r) => {
          if (!r.ok) throw new Error(`${r.status}`);
          return r.arrayBuffer();
        })
        .then((buf) => {
          const name = fileUrl.split("/").pop() || "remote.dwg";
          loadBytesAsTab(name, new Uint8Array(buf));
        })
        .catch((e) => console.error(`[CodeCAD] ?file= load failed: ${e}`));
    }

    // Optional server-configured comparison: open the second drawing in its
    // own synced session and show both canvases at once.
    const compareDocId = (
      window as Window & { __CADVIEW_COMPARE_DOC_ID?: string }
    ).__CADVIEW_COMPARE_DOC_ID;
    if (compareDocId && compareDocId !== initId) {
      cad.api.listDocuments().then((docs) => {
        if (!docs.some((doc) => doc.id === compareDocId)) return;
        cad.sessions.create(compareDocId);
        setTabs((prev) => prev.some((tab) => tab.id === compareDocId)
          ? prev
          : [...prev, {
              id: compareDocId,
              label: compareDocId.replace(/\.dwg$/i, "").split("/").pop() || compareDocId,
            }]);
        setCompareTabId(compareDocId);
      }).catch((e) => console.error(`[CodeCAD] Compare setup failed: ${e}`));
    }
  }, []);

  // Switch active tab
  function activateTab(id: string) {
    if (id !== comparePrimaryId && id !== compareTabId) setCompareTabId(null);
    setActiveTabId(id);
    try {
      cad.useSession(id);
    } catch {
      // Session may not exist yet
    }
  }

  // Open a server document
  function openDocument(docId: string) {
    if (tabs.some((tab) => tab.id === docId)) {
      activateTab(docId);
      return;
    }
    const label =
      docId
        .replace(/\.dwg$/i, "")
        .split("/")
        .pop() || docId;
    try {
      cad.sessions.create(docId);
    } catch (e) {
      console.warn("Failed to create session:", e);
      return;
    }
    setTabs((prev) => [...prev, { id: docId, label }]);
    activateTab(docId);
  }

  // New empty drawing
  function newDrawing() {
    _newDrawingCounter++;
    const id = `new-${Date.now()}-${_newDrawingCounter}`;
    const label = `untitled-${_newDrawingCounter}`;
    try {
      cad.sessions.create(id);
    } catch (e) {
      console.warn("Failed to create new drawing:", e);
      return;
    }
    setTabs((prev) => [...prev, { id, label }]);
    activateTab(id);
  }

  // Load DWG bytes into a new tab
  function loadBytesAsTab(name: string, bytes: Uint8Array) {
    const label = name.replace(/\.dwg$/i, "");
    const id = `file-${Date.now()}-${name}`;
    try {
      cad.sessions.create(id);
    } catch (e) {
      console.warn("Failed to create session:", e);
      return;
    }
    setTabs((prev) => [...prev, { id, label }]);
    activateTab(id);
    try {
      const result = cad.sessions.loadDwgBytes(id, bytes);
      console.log(`[CodeCAD] Loaded ${name}: ${result.entities} entities`);
    } catch (e) {
      console.error(`[CodeCAD] Failed to parse ${name}:`, e);
    }
  }

  // Load a dropped DWG file into a new tab
  function loadDroppedFile(file: File) {
    file.arrayBuffer().then((buf) => {
      loadBytesAsTab(file.name, new Uint8Array(buf));
    });
  }

  // Load an example drawing by ID
  function loadExample(exampleId: string) {
    const url = exampleDwgUrl(exampleId);
    fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`${r.status}`);
        return r.arrayBuffer();
      })
      .then((buf) => loadBytesAsTab(`${exampleId}.dwg`, new Uint8Array(buf)))
      .catch((e) => console.error(`[CodeCAD] Failed to load example: ${e}`));
  }

  // Close a tab
  function closeTab(id: string) {
    if (id === compareTabId || id === comparePrimaryId) setCompareTabId(null);
    try {
      cad.sessions.destroy(id);
    } catch {
      // Already destroyed
    }
    setTabs((prev) => {
      const next = prev.filter((t) => t.id !== id);
      if (activeTabId === id && next.length > 0) {
        activateTab(next[next.length - 1].id);
      }
      return next;
    });
  }

  // Drag and drop handlers
  const onDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer.types.includes("Files")) {
      setDragOver(true);
    }
  }, []);

  const onDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
  }, []);

  // biome-ignore lint/correctness/useExhaustiveDependencies: stable ref
  const onDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragOver(false);
      const files = Array.from(e.dataTransfer.files);
      for (const file of files) {
        if (file.name.toLowerCase().endsWith(".dwg")) {
          loadDroppedFile(file);
        } else {
          console.warn(`[CodeCAD] Ignoring non-DWG file: ${file.name}`);
        }
      }
    },
    [tabs],
  );

  // Poll layers from the active session
  // biome-ignore lint/correctness/useExhaustiveDependencies: restart poll on tab switch
  useEffect(() => {
    const poll = setInterval(() => {
      try {
        const entities = cad.entities() as EntityJson[];
        const layerMap = new Map<
          string,
          { color: [number, number, number]; count: number }
        >();
        for (const e of entities) {
          const existing = layerMap.get(e.layer);
          if (existing) {
            existing.count++;
          } else {
            layerMap.set(e.layer, { color: e.color, count: 1 });
          }
        }
        setLayers((prev) => {
          const next: LayerState[] = [];
          for (const [name, info] of layerMap) {
            const existing = prev.find((l) => l.name === name);
            next.push({
              name,
              color: info.color,
              visible: existing?.visible ?? true,
              entityCount: info.count,
            });
          }
          next.sort((a, b) => a.name.localeCompare(b.name));
          return next;
        });
      } catch {
        // WASM not ready yet
      }
    }, 1000);
    return () => clearInterval(poll);
  }, [activeTabId]);

  const activeTab = tabs.find((t) => t.id === activeTabId);

  return (
    <div
      className="app-root"
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
    >
      <TabBar
        tabs={tabs}
        activeId={activeTabId}
        onSelect={activateTab}
        onClose={closeTab}
        onAdd={() => setPickerOpen(true)}
        onCompare={() => {
          if (compareTabId) setCompareTabId(null);
          else {
            const other = tabs.find((tab) => tab.id !== activeTabId);
            if (other) {
              setComparePrimaryId(activeTabId);
              setCompareTabId(other.id);
            }
            else setPickerOpen(true);
          }
        }}
        comparing={compareTabId !== null}
        markupMode={markupMode}
        onToggleMarkup={() => setMarkupMode((value) => !value)}
        onClearMarkup={() => {
          try {
            cad.useSession(activeTabId);
            const marks = cad.entities({ layer: "E_FEEDBACK_MARKS" });
            if (marks.length) cad.remove(marks);
          } catch (error) {
            console.error("[CodeCAD] Could not clear feedback marks:", error);
          }
        }}
      />

      <div className={`viewport-area${compareTabId ? " compare" : ""}`}>
        {compareTabId ? [comparePrimaryId, compareTabId].map((id) => {
          const tab = tabs.find((candidate) => candidate.id === id);
          if (!tab) return null;
          return <div className="compare-pane" key={id}>
            <div className="compare-header">{tab.label}</div>
            <ViewportContainer sessionId={id} focused={activeTabId === id} markupMode={markupMode}
              onFocus={() => activateTab(id)} />
          </div>;
        }) : activeTab && (
          <ViewportContainer key={activeTab.id} sessionId={activeTab.id}
            focused={true} markupMode={markupMode} onFocus={() => {}} />
        )}
        {dragOver && (
          <div className="drop-overlay">
            <div className="drop-message">Drop .dwg file to open</div>
          </div>
        )}

        {layers.length > 0 && (
          <LayerPanel
            layers={layers}
            open={panelOpen}
            onTogglePanel={() => setPanelOpen(!panelOpen)}
            onToggleLayer={(name) => {
              setLayers((prev) =>
                prev.map((l) => {
                  if (l.name !== name) return l;
                  return { ...l, visible: !l.visible };
                }),
              );
              const cur = layers.find((l) => l.name === name);
              cad.setLayerVisible(name, !(cur?.visible ?? true));
            }}
          />
        )}
      </div>

      <DocumentPicker
        open={pickerOpen}
        onClose={() => setPickerOpen(false)}
        onSelect={openDocument}
        onNewDrawing={newDrawing}
        onLoadFile={loadBytesAsTab}
        onLoadExample={loadExample}
        openTabIds={tabs.map((t) => t.id)}
      />
    </div>
  );
}
