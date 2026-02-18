import Globe from './components/Globe';
import VoiceButton from './components/VoiceButton';
import './App.css';

function App() {
  return (
    <div className="app">
      <header className="hud-header">
        <span className="title">Globalwatch</span>
        <span className="status">ACTIVE MONITORING // 6 EVENTS TRACKED</span>
      </header>

      <Globe />
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
