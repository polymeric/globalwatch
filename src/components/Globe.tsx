import { useEffect, useRef, useCallback } from 'react';
import { initGlobeScene, type GlobeScene } from '../globe/scene';
import type { HoverData } from '../globe/interactions';
import { getEvents } from '../lib/tauriEvents';
import { useState } from 'react';

const REFRESH_MS = 5 * 60 * 1000; // 5 minutes

interface Props {
  onEventCount?: (n: number) => void;
}

export default function Globe({ onEventCount }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<GlobeScene | null>(null);
  const [tooltip, setTooltip] = useState<HoverData>(null);

  const handleHover = useCallback((data: HoverData) => {
    setTooltip(data);
  }, []);

  // Initialise the Three.js scene once.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const globe = initGlobeScene(el, handleHover);
    sceneRef.current = globe;
    return () => {
      globe.destroy();
      sceneRef.current = null;
    };
  }, [handleHover]);

  // Fetch events from the Rust backend and push them to the scene.
  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const events = await getEvents();
        if (cancelled) return;
        sceneRef.current?.updateMarkers(events);
        onEventCount?.(events.length);
        console.log(`[events] loaded ${events.length} events`);
      } catch (err) {
        console.warn('[events] fetch failed:', err);
      }
    }

    load();
    const id = setInterval(load, REFRESH_MS);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [onEventCount]);

  return (
    <div ref={containerRef} className="globe-container">
      {tooltip && (
        <div
          className="marker-tooltip"
          style={{ left: tooltip.x + 14, top: tooltip.y - 12 }}
        >
          <div className="tooltip-label">{tooltip.label}</div>
          <div className="tooltip-headline">{tooltip.headline}</div>
          <div className="tooltip-meta">
            <span className={`tooltip-tag tooltip-tag--${tooltip.category}`}>
              {tooltip.category}
            </span>
            <span className={`tooltip-tag tooltip-tag--${tooltip.severity}`}>
              {tooltip.severity}
            </span>
            <span className="tooltip-source">{tooltip.source}</span>
          </div>
        </div>
      )}
    </div>
  );
}
