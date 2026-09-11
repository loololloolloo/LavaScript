# LavaScript

LavaScript is a new programming language with a Lua + JavaScript-inspired syntax and a native WebAssembly compiler backend.

## Current status

The repository now contains the first compiler bootstrap:

```text
LavaScript source (.ls)
        |
        v
     compiler
        |
        v
WebAssembly (.wasm)
```

The first supported statement is:

```lavascript
print 42
```

`print` is compiled to a WebAssembly call to the host import `lavascript.print_i32`. The generated module also exports `main`.

## Build

Install Rust, then:

```bash
cargo run -- build examples/hello.ls -o hello.wasm
```

Run the compiler tests with:

```bash
cargo test
```

## Roadmap

- [x] Compiler executable
- [x] WebAssembly module generation
- [x] Host `print_i32` import
- [x] `.ls` example
- [ ] Lexer
- [ ] Parser and AST
- [ ] Arithmetic expressions
- [ ] Variables
- [ ] `if` / `while`
- [ ] Functions
- [ ] Objects and arrays
- [ ] Browser runtime
- [ ] Optimizations
