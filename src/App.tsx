import { useState, useCallback } from 'react';
import Globe from './components/Globe';
import VoiceButton from './components/VoiceButton';
import './App.css';

function App() {
  const [eventCount, setEventCount] = useState<number | null>(null);

  const handleEventCount = useCallback((n: number) => {
    setEventCount(n);
  }, []);

  const statusText =
    eventCount === null
      ? 'ACTIVE MONITORING'
      : `ACTIVE MONITORING // ${eventCount} EVENTS TRACKED`;

  return (
    <div className="app">
      <header className="hud-header">
        <span className="title">Globalwatch</span>
        <span className="status">{statusText}</span>
      </header>

      <Globe onEventCount={handleEventCount} />
      <VoiceButton />

      <footer className="hud-status">
        <span className="left">
          <span className="indicator" />
          SYSTEM ONLINE — CLICK GLOBE TO PAUSE / DRAG TO ROTATE
        </span>
        <span className="right">LAT 0.00 LON 0.00 // ZOOM 1.0x</span>
      </footer>
    </div>
  );
}

export default App;
