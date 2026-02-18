# globalwatch

A desktop application that displays an interactive 3D globe with a green-on-black matrix-style theme. It renders country outlines, lat/lon grid lines, and triangular markers for dangerous weather patterns, crises, and events sourced from news feeds. Includes a voice-powered AI assistant interface.

## Features

- Interactive 3D globe with auto-spin (click to pause, drag to rotate/tilt)
- Green monochrome matrix aesthetic on black background
- Country outlines rendered from TopoJSON data
- Triangular event markers with severity-based brightness
- Marker tooltips with source, timestamp, and severity on hover/click
- Marker lifecycle management (appear, update, expire)
- Push-to-talk voice interface (STT → AI assistant → TTS)
  - Speech-to-text via [whisper.cpp](https://github.com/ggerganov/whisper.cpp) (native Rust bindings)
  - AI assistant via local LLM (LM Studio / any OpenAI-compatible endpoint)
  - Text-to-speech via native macOS `say` command (enhanced voices supported)
  - Conversation history with context carry-over
- 60 FPS rendering target

## Prerequisites

- [Node.js](https://nodejs.org/) (LTS recommended)
- [Rust](https://www.rust-lang.org/tools/install) 1.77.2+
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform (system libs, Xcode CLI tools on macOS, etc.)
- CMake (required to build whisper.cpp — install via `brew install cmake` on macOS)

## Getting Started

```bash
# Install frontend dependencies
make install

# Launch the app in dev mode (Tauri window + Vite HMR)
make dev
```

### Voice Assistant Setup

The voice assistant requires a Whisper model and a running LLM server.

**1. Download the Whisper model**

Download `ggml-base.en.bin` (~148 MB) from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp/tree/main) and place it in the app's models directory:

```
# macOS
~/Library/Application Support/com.globalwatch.app/models/ggml-base.en.bin

# Linux
~/.local/share/com.globalwatch.app/models/ggml-base.en.bin
```

The app will show the exact path and a setup prompt if the model is missing.

**2. Start a local LLM**

The assistant sends queries to an OpenAI-compatible chat completions endpoint at `http://localhost:1234/v1/chat/completions`. Any of these will work:

- [LM Studio](https://lmstudio.ai/) — load any chat model and enable the local server
- [Ollama](https://ollama.com/) — run with `OLLAMA_HOST=localhost:1234 ollama serve`
- Any OpenAI-compatible API server on port 1234

**3. Use the voice button**

Hold the voice button in the bottom-right corner of the app, speak your question, and release. The pipeline runs: mic capture → Whisper transcription → LLM response → spoken reply via TTS.

## Make Targets

| Command | Description |
|---|---|
| `make install` | Install npm dependencies |
| `make dev` | Launch Tauri app with Vite HMR |
| `make dev-voice` | Launch with microphone support (codesigned for macOS) |
| `make dev-web` | Frontend only in browser (no Tauri shell) |
| `make build` | Full production build |
| `make package` | Production build with platform bundle (.app/.dmg, .AppImage/.deb) |
| `make lint` | Run ESLint + cargo clippy |
| `make test` | Run Rust tests |
| `make clean` | Remove dist, Vite cache, and Rust target directory |

## Project Structure

```
src/                    # React/TypeScript frontend
├── components/         # React components
│   ├── Globe.tsx       #   Globe viewer wrapper
│   ├── VoiceButton.tsx #   Push-to-talk voice UI
│   └── VoiceButton.css #   Matrix-themed voice button styles
├── lib/
│   ├── audioUtils.ts   #   Audio utilities, WAV playback helpers
│   └── tauriVoice.ts   #   Typed wrappers for Tauri voice commands
├── globe/              # Three.js globe modules
│   ├── scene.ts        #   Scene, camera, renderer setup
│   ├── materials.ts    #   Shared green monochrome materials
│   ├── wireframe.ts    #   Lat/lon grid lines
│   ├── countries.ts    #   Country outlines from TopoJSON
│   ├── markers.ts      #   Event marker rendering + lifecycle
│   └── interactions.ts #   Mouse/touch controls
├── data/               # Static data assets (TopoJSON, etc.)
└── main.tsx            # App entry point

src-tauri/              # Tauri 2 / Rust backend
├── src/
│   ├── main.rs         # Tauri app bootstrap
│   ├── lib.rs          # Commands, plugins, state management
│   └── voice/          # Voice pipeline modules
│       ├── capture.rs  #   Native mic capture via cpal
│       ├── stt.rs      #   Whisper speech-to-text
│       ├── llm.rs      #   LM Studio / OpenAI-compatible chat client
│       ├── tts.rs      #   Native macOS text-to-speech
│       └── models.rs   #   Model file path management
├── tauri.conf.json     # App identity, window config, bundling
└── Cargo.toml          # Rust dependencies
```

## Tech Stack

- **App shell**: [Tauri 2](https://v2.tauri.app/) (Rust backend) + React 19 + TypeScript
- **Bundler**: [Vite](https://vite.dev/)
- **3D rendering**: [Three.js](https://threejs.org/)
- **Map data**: [TopoJSON](https://github.com/topojson/topojson)
- **Speech-to-text**: [whisper-rs](https://github.com/tazz4843/whisper-rs) (whisper.cpp Rust bindings)
- **LLM inference**: Local via [LM Studio](https://lmstudio.ai/) or [Ollama](https://ollama.com/) (OpenAI-compatible API)
- **Text-to-speech**: Native macOS `say` command (enhanced system voices)
- **Audio capture**: [cpal](https://github.com/RustAudioGroup/cpal) (native cross-platform audio I/O)

## Voice Pipeline

```
Hold button → Native mic capture (cpal, Rust backend)
    → Downsample to 16kHz mono
    → whisper-rs transcription
    → HTTP POST to LLM (localhost:1234)
    → Response text displayed + spoken via native TTS
```

## Data Flow

1. Backend polls news/weather APIs on a schedule
2. Events are classified by severity and geocoded
3. Results are stored in SQLite
4. Frontend fetches active events via Tauri commands
5. Markers are rendered on the globe with appropriate severity styling

## Packaging

```bash
make package
```

This produces platform-specific bundles:
- **macOS**: `.app` and `.dmg` in `src-tauri/target/release/bundle/`
- **Linux**: `.AppImage` and `.deb` in `src-tauri/target/release/bundle/`

## License

[MIT](LICENSE)
