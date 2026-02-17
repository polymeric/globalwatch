import { useEffect, useRef } from 'react';
import { initGlobeScene } from '../globe/scene';

export default function Globe() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const globe = initGlobeScene(el);
    return () => globe.destroy();
  }, []);

  return <div ref={containerRef} className="globe-container" />;
}
