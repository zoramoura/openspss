# Web application

The web client is a Vue 3, TypeScript, and Vite scaffold. The current UI still displays the Vue/Vite starter screen. A Pinia store and Comlink Web Worker are available for calling the Rust/WebAssembly descriptive-statistics engine, but are not yet connected to a user-facing workflow.

For requirements, setup, and repository status, see the [project README](../README.md).

Run the development server from the repository root with:

```bash
npm run dev
```

Build the WASM engine and web client from the repository root with:

```bash
npm run build
```
