# gb-web
## Project Overview

*   **Goal:** Provide a reasonably accurate, performant, and feature-rich Game Boy emulator in the browser.
*   **Core Emulation:** Written in Rust, handling CPU, PPU, APU, and Memory.
*   **Frontend:** Built with Svelte 5 and TypeScript, providing the UI for controls, settings, file management, and rendering.
*   **WASM Bridge:** The Rust core is exposed to the frontend via `wasm-bindgen`, utilizing `wgpu` for rendering and `cpal` for audio.

## Tech Stack

*   **Frontend:**
    *   **Framework:** Svelte 5
    *   **Language:** TypeScript
    *   **Build Tool:** Vite
    *   **State Management:** Svelte Runes (`.svelte.ts`)
    *   **Styling:** Raw CSS / Svelte scoped styles
*   **Backend (Core):**
    *   **Language:** Rust (Edition 2024)
    *   **Compilation:** WebAssembly (`wasm32-unknown-unknown`)
    *   **Tooling:** `wasm-pack`
    *   **Graphics:** `wgpu` (using the WebGL backend for browser)
    *   **Audio:** `cpal`
*   **Tools:**
    *   **Scraper:** Python script to fetch ROM metadata from Homebrew Hub.

## Directory Structure

*   **`app/`**: Static assets and public files.
    *   `assets/`: Images, fonts, icons.
    *   `public/`: Public root for the web server.
    *   `roms/`: ROM metadata (in `json`) and the `scraper.py` tool.
*   **`src/`**: Svelte frontend source code.
    *   `App.svelte`: Main application component.
    *   `bridge.svelte.ts`: Interface between Svelte and the WASM core.
    *   `db.svelte.ts`: IndexedDB interaction (via `dexie`).
*   **`core/`**: Platform-agnostic Game Boy emulation logic.
    *   The `CPU` class (defined in `cpu/`) is the emulator's main class that owns and communicates with all the other components.
    *   Other `.rs` files are the emulation components.
*   **`wasm/`**: WASM-specific platform implementation.
    *   `src/lib.rs`: Entry point for `wasm-bindgen`.
    *   `src/renderer/`: WGPU rendering logic. The shader code is written in WGSL.
    *   `src/audio.rs`: Audio handling.
    *   `src/proxy.rs`: Rust's side of the interface between Svelte and WASM.
*   **`debugger/`**: A native Rust CLI debugger that is separate from the web app. It simply runs the `core` module locally instead in WASM, useful for breakpoints and other debugging.

## Development Conventions
*   **Rust vs. JS:** Performance-critical emulation code resides in `core` and `wasm` (Rust). UI and user interaction logic reside in `src` (Svelte/TS).
*   **State Management:** Svelte 5 runes (`$state`, `$effect`) are used for reactive state management in `.svelte.ts` files (e.g., `options.svelte.ts`).
*   **Styling:** The UI uses a simple single colored pixelated aesthetic in CSS and assets.
*   **WASM Integration:** `vite-plugin-wasm` and `vite-plugin-top-level-await` are used to handle WASM modules in the browser.

# Instructions
@./.gemini/instructions.md
