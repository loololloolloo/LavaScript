# Vortex WASM port

This directory is the active browser-port track for the supplied `Vortex.exe` binary.

The target is the original Vortex runtime and recovered assets, not a visual recreation.

## Runtime

- Bevy `0.19.1`
- `wasm32-unknown-unknown`
- Bevy's native browser window/render integration
- HTML canvas selector: `#vortex-canvas`
- Asset metadata disabled because extracted assets are runtime artifacts rather than `.meta`-managed project files

## Actual recovered asset path

The supplied executable contains the R7 GLB hierarchy. The extraction pipeline writes the recovered model to:

```text
vortex-wasm/assets/vortex-r7.glb
```

The Bevy entry point loads that file with `AssetServer` and spawns glTF scene 0. No procedural avatar geometry is used by the runtime.

## Browser build

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/vortex_wasm.wasm --target web --out-dir dist
```

Copy the generated `vortex-wasm/assets` directory into the web server's `dist/assets` directory and serve `dist` over HTTP(S).

## Important asset boundary

The repository contains the extraction tooling, not an assumed copy of proprietary binary assets. Run the extractor against the supplied executable to materialize the actual GLB/KTX2/Ogg resources locally before packaging the browser build.

## Reverse-engineering track

`tools/vortex_pdata.py` enumerates x64 exception/unwind ranges from `.pdata`. The next mapping pass correlates those ranges with embedded Rust/Bevy symbol-like strings and source-path markers recovered from the executable. The goal is to separate Bevy framework functions from Vortex gameplay functions before porting behavior.
