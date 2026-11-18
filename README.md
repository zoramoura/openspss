# OpenSPSS

A work-in-progress statistical computing application. The repository currently contains a Vue 3 web scaffold and a small Rust/WebAssembly engine with a Comlink worker bridge. The application UI is still the default Vue/Vite starter screen; the statistical worker is not yet connected to a user-facing analysis workflow.

## Current implementation

- **Web app:** Vue 3, TypeScript, and Vite, with Pinia registered.
- **Statistics worker:** a dedicated Web Worker exposed through Comlink. Its Pinia store initializes the engine lazily and provides a `getDescriptives(numbers)` action.
- **WASM engine:** Rust compiled with `wasm-pack` and `wasm-bindgen`. The current `compute_descriptives` function returns:
  - `count`
  - `mean`
  - `variance` (population variance, calculated with divisor `N`)
  - `std_dev`
  - `min`
  - `max`
- An empty input returns an error.

The worker and WASM calculation have been manually exercised in a browser. This does not constitute a statistical validation suite or a performance guarantee.

## Repository layout

```text
.
├── package.json              # Root commands forwarding to the web app
├── engine/
│   ├── Cargo.toml
│   └── src/lib.rs            # Current descriptive-statistics WASM export
└── web/
    ├── package.json
    ├── vite.config.ts
    └── src/
        ├── App.vue           # Currently renders the starter HelloWorld view
        ├── main.ts           # Vue app and Pinia setup
        ├── stores/stats.ts   # Comlink worker bridge
        ├── workers/stats.worker.ts
        └── wasm/             # wasm-pack generated module and binary
```

## Requirements

- Node.js `^20.19.0` or `>=22.12.0` (required by the installed Vite 8 release)
- npm
- Rust stable and Cargo
- `wasm-pack`
- The Rust `wasm32-unknown-unknown` target

Install the Rust target and `wasm-pack` if needed:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Setup and development

From the repository root, install the web dependencies:

```bash
npm --prefix web install
```

Build the WASM engine and web app:

```bash
npm run build
```

The build command compiles the engine to `web/src/wasm` and then runs the Vite production build. To start the development server from the repository root:

```bash
npm run dev
```

The web package also provides `dev`, `build`, and `build:wasm` scripts. If running commands directly in `web/`, first generate the WASM package with `npm run build:wasm` when it is not already present.

## Not implemented yet

The repository does not currently include the planned data grid, dataset import/export, inferential tests, regression or ANOVA routines, SPSS `.sav`/`.zsav` support, statistical benchmark tests, Docker deployment, or a Tauri desktop application. The UI remains the starter template, and the worker-backed statistics store is not yet used by that UI.

These are potential future features, not capabilities of the current application. In particular, claims about regulatory compliance, data privacy guarantees, bundle size, and frame rate have not been established.
