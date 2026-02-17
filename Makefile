.PHONY: dev build clean lint test package install

# Install all dependencies
install:
	npm install

# Run in dev mode (Tauri window + Vite HMR)
dev:
	npm run tauri dev

# Run frontend only in browser (no Tauri)
dev-web:
	npm run dev

# Build everything (frontend + Rust backend)
build:
	npm run tauri build

# Lint frontend and Rust
lint:
	npm run lint
	cd src-tauri && cargo clippy

# Run Rust tests
test:
	cd src-tauri && cargo test

# Clean all build artifacts
clean:
	rm -rf dist node_modules/.vite
	cd src-tauri && cargo clean

# Production package (alias for build)
package: build
