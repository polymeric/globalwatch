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
- 60 FPS rendering target

## Prerequisites

- [Node.js](https://nodejs.org/) (LTS recommended)
- [Rust](https://www.rust-lang.org/tools/install) 1.77.2+
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform (system libs, Xcode CLI tools on macOS, etc.)

## Getting Started

```bash
# Install frontend dependencies
make install

# Launch the app in dev mode (Tauri window + Vite HMR)
make dev
```

## Make Targets

| Command | Description |
|---|---|
| `make install` | Install npm dependencies |
| `make dev` | Launch Tauri app with Vite HMR |
| `make dev-web` | Frontend only in browser (no Tauri shell) |
| `make build` | Full production build |
| `make package` | Production build with platform bundle (.app/.dmg, .AppImage/.deb) |
| `make lint` | Run ESLint + cargo clippy |
| `make test` | Run Rust tests |
| `make clean` | Remove dist, Vite cache, and Rust target directory |

## Project Structure

```
src/                    # React/TypeScript frontend
├── components/         # React components (Globe viewer)
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
│   └── lib.rs          # Commands, plugins, backend logic
├── tauri.conf.json     # App identity, window config, bundling
└── Cargo.toml          # Rust dependencies
```

## Tech Stack

- **App shell**: [Tauri 2](https://v2.tauri.app/) (Rust backend) + React 19 + TypeScript
- **Bundler**: [Vite](https://vite.dev/)
- **3D rendering**: [Three.js](https://threejs.org/)
- **Map data**: [TopoJSON](https://github.com/topojson/topojson)

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
