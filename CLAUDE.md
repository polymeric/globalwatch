# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Globalwatch is a desktop application that displays an interactive 3D globe with a green-on-black matrix-style theme. It shows country outlines, lat/lon grid lines, and triangular markers for dangerous weather patterns, crises, or events sourced from news feeds. Includes a voice-powered AI assistant interface.

## Tech Stack

- **App shell**: Tauri 2 (Rust backend) + React 19 + TypeScript (Vite)
- **3D rendering**: Three.js — black background, green monochrome palette
- **Local storage**: SQLite (event cache, user settings, voice transcripts)
- **Packaging targets**: macOS (.app/.dmg), Linux (.AppImage/.deb)

## Build & Dev Commands

```bash
npm install                  # Install frontend dependencies
npm run tauri dev            # Dev mode — launches Tauri window with Vite HMR
npm run tauri build          # Production build (creates platform bundle)
npm run dev                  # Frontend-only dev server (browser, no Tauri)
npm run build                # Build frontend only (tsc + vite build)
npm run lint                 # ESLint
```

Rust backend (run from `src-tauri/`):
```bash
cargo build                  # Build Rust backend
cargo test                   # Run Rust tests
cargo clippy                 # Lint Rust code
```

**Note:** Rust 1.87 is current — `time` crate is pinned in Cargo.lock to 0.3.45 for compatibility.

## Architecture

- **Frontend (`src/`)**: React/TS app — 3D globe scene via Three.js, event marker rendering, voice UI controls
- **Backend (`src-tauri/src/`)**: Tauri Rust backend — OS integration (microphone, keychain for API keys, background tasks), news/weather API polling worker, event classifier, geocoder, SQLite persistence
- **Tauri config**: `src-tauri/tauri.conf.json` — app identity, window settings, bundling
- **Data flow**: Backend polls news APIs → classifies/geocodes events → stores in SQLite → frontend fetches active events via Tauri commands → renders markers on globe

## Key Features

- Globe auto-spins; click to pause, click-drag to rotate/tilt
- Triangular event markers with severity variants (green palette brightness)
- Markers have hover/click tooltips (source, timestamp, severity) and lifecycle (appear, update, expire)
- Push-to-talk voice interface: STT → AI assistant → TTS response
- 60 FPS rendering target
