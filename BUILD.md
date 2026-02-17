Globalwatch Build Plan

Define scope and acceptance criteria (Day 1)

Confirm MVP features from README.md: 3D globe, matrix-style theme, spin/pause, drag rotate/tilt, event markers, voice AI interface.

Write acceptance checks for each feature on both macOS and Linux.

Decide first release constraints: desktop app, single-user, English voice, near-real-time news polling.

Choose stack and app shell (Day 1-2)

Use Tauri + React + TypeScript for cross-platform desktop packaging (macOS + Linux) with lower resource use than Electron.

Use Three.js for globe rendering.

Use Tauri Rust backend for OS integration (microphone, local storage, secure keys, background tasks).

Use SQLite for local persistence (events cache, user settings, transcripts).

Globe rendering foundation (Week 1)

Build 3D scene with black background, green monochrome material palette.

Add country outlines and lat/lon grid layers.

Implement default globe spin animation.

Add mouse click to pause/resume spin.

Add click-drag rotation/tilt with smooth damping and constraints.

Event ingestion pipeline (Week 2)

Create backend worker to poll news/weather APIs on a schedule.

Add classifier rules (keyword/risk score + location extraction).

Geocode event locations to lat/lon.

Store normalized events in SQLite.

Expose frontend API to fetch active events.

Marker visualization (Week 2)

Render triangular markers at event coordinates.

Add marker severity color/intensity variants (still green palette, brightness differences).

Add hover/click tooltip with source, timestamp, severity.

Add marker lifecycle: appear, update, expire.

Voice + AI assistant (Week 3)

Implement push-to-talk or wake-button flow.

Speech-to-text pipeline (local or cloud provider abstraction).

AI assistant endpoint for Q&A/actions over current globe state.

Text-to-speech response playback.

Add privacy controls (mic permission, transcript retention toggle).

Reliability, security, and UX hardening (Week 4)

API key management via OS keychain/secure storage.

Network failure handling, retries, and offline state.

Performance optimization to keep 60 FPS target on mid-tier hardware.

Accessibility basics: keyboard controls, readable contrast, reduced-motion option.

Testing and release

Unit tests: event normalization, geocoding mapping, marker lifecycle.

Integration tests: ingest -> DB -> marker render path.

Manual cross-platform QA matrix: macOS (Apple Silicon + Intel if possible), Ubuntu LTS.

Build signed release artifacts: .app/.dmg for macOS, .AppImage/.deb for Linux.

Publish v0.1 with known limitations and telemetry-free default mode.

Suggested MVP Milestones

Milestone A: Interactive globe only.
Milestone B: Live event markers from one news source.
Milestone C: Voice assistant with basic Q&A.
Milestone D: Production packaging and QA.
Key Risks to manage early

Accurate location extraction from news text.
Voice stack differences across macOS/Linux.
Rendering performance with many markers.
API rate limits and source reliability.
