# LavaScript WebAssembly runtime

This folder contains a minimal browser host for LavaScript's WebAssembly output.

The compiler imports `lavascript.print_i32`. The browser supplies that function and writes values to the page and console.

To run a compiled module, put it next to `index.html` as `out.wasm` and serve the folder over HTTP. Browsers generally block `fetch()` for local `file://` pages.
