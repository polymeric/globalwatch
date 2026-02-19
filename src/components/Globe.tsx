import { useEffect, useRef, useState, useCallback } from 'react';
import { initGlobeScene } from '../globe/scene';
import type { HoverData } from '../globe/interactions';

export default function Globe() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [tooltip, setTooltip] = useState<HoverData>(null);

  const handleHover = useCallback((data: HoverData) => {
    setTooltip(data);
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const globe = initGlobeScene(el, handleHover);
    return () => globe.destroy();
  }, [handleHover]);

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
