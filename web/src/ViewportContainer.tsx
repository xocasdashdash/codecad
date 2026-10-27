import { useEffect, useRef, useState } from "react";
import { cad } from "./cad";

interface Props {
  sessionId: string;
  focused: boolean;
  markupMode: boolean;
  onFocus: () => void;
}

type Point = { x: number; y: number };

export function ViewportContainer({ sessionId, focused, markupMode, onFocus }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererKeyRef = useRef<string>("");
  const [stroke, setStroke] = useState<Point[]>([]);
  const strokeRef = useRef<Point[]>([]);
  const straightRef = useRef(false);

  function pointAt(event: React.PointerEvent<SVGSVGElement>): Point {
    const rect = event.currentTarget.getBoundingClientRect();
    return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }

  function previewPoints(event?: React.PointerEvent<SVGSVGElement>): Point[] {
    const points = strokeRef.current;
    if (points.length < 2) return points;
    if (straightRef.current || event?.shiftKey) return [points[0], points[points.length - 1]];
    return points;
  }

  function startStroke(event: React.PointerEvent<SVGSVGElement>) {
    if (!focused || !markupMode || event.button !== 0) return;
    event.preventDefault();
    onFocus();
    event.currentTarget.setPointerCapture(event.pointerId);
    straightRef.current = event.shiftKey;
    strokeRef.current = [pointAt(event)];
    setStroke(strokeRef.current);
  }

  function moveStroke(event: React.PointerEvent<SVGSVGElement>) {
    if (!strokeRef.current.length) return;
    if (event.shiftKey) straightRef.current = true;
    const p = pointAt(event);
    const points = strokeRef.current;
    if (points.length === 1) strokeRef.current = [...points, p];
    else strokeRef.current = [...points, p];
    setStroke(strokeRef.current);
  }

  function finishStroke(event: React.PointerEvent<SVGSVGElement>) {
    if (!strokeRef.current.length) return;
    const p = pointAt(event);
    let points = [...strokeRef.current];
    if (points.length === 1 || points.at(-1)?.x !== p.x || points.at(-1)?.y !== p.y) points.push(p);
    if (straightRef.current || event.shiftKey) points = [points[0], points[points.length - 1]];
    if (points.length > 1) {
      try {
        cad.useSession(sessionId);
        const pixelRatio = cad.viewport.rendererType === "vello" ? window.devicePixelRatio : 1;
        const worldPoints = points.map(({ x, y }) => cad.screenToWorld(sessionId, x, y, pixelRatio));
        cad.addPolyline(worldPoints, { layer: "E_FEEDBACK_MARKS", color: [255, 64, 160], lineweight: 0.8 });
      } catch (error) {
        console.error("[CodeCAD] Could not save markup stroke:", error);
      }
    }
    strokeRef.current = [];
    setStroke([]);
  }

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    try {
      rendererKeyRef.current = cad.viewport.start(canvas, sessionId);
    } catch (e) {
      console.error(`[CodeCAD] Failed to start renderer for ${sessionId}:`, e);
    }

    return () => {
      try {
        cad.viewport.stop(rendererKeyRef.current);
      } catch {
        // Session may already be destroyed
      }
    };
  }, [sessionId]);

  return (
    <div
      className={`viewport-container${focused ? " focused" : ""}${markupMode && focused ? " markup-active" : ""}`}
      onClick={onFocus}
    >
      <canvas ref={canvasRef} />
      {markupMode && focused && (
        <svg className="markup-overlay" onPointerDown={startStroke} onPointerMove={moveStroke}
          onPointerUp={finishStroke} onPointerCancel={finishStroke}>
          {stroke.length > 0 && <polyline
            points={previewPoints().map((p) => `${p.x},${p.y}`).join(" ")}
            fill="none" stroke="#ff40a0" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />}
        </svg>
      )}
    </div>
  );
}
