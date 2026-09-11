# LavaScript

LavaScript is a new programming language with a Lua + JavaScript-inspired syntax and a native WebAssembly compiler backend.

## Compiler pipeline

```text
LavaScript source (.ls)
        |
        v
      Lexer
        |
        v
   Parser / AST
        |
        v
 WebAssembly codegen
        |
        v
WebAssembly (.wasm)
        |
        v
 Browser runtime
```

## Current language features

The compiler bootstrap now supports:

- integer numbers and arithmetic: `+`, `-`, `*`, `/`, `%`
- variables with `let` and reassignment
- booleans: `true`, `false`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- logical operators: `and`, `or`, `not`
- JavaScript-style logical spellings: `&&`, `||`, `!`
- `if`, `elseif`, `else`, and `end`
- `while` loops
- `break`
- functions with parameters and return values
- string literals with `\\n`, `\\r`, `\\t`, `\\"`, and `\\\\` escapes
- `print` for integers, booleans, and string literals
- `--` and `//` single-line comments
- WebAssembly linear memory for string data
- browser execution through the included runtime

Example:

```lavascript
function clamp(value, low, high)
    if value < low then
        return low
    elseif value > high then
        return high
    end
    return value
end

let x = 12
let enabled = true

while enabled
    print clamp(x, 0, 10)
    x = x - 1
    if x == 0 then
        enabled = false
    end
end
```

## Build

Install Rust, then:

```bash
cargo run -- build examples/hello.ls -o hello.wasm
```

Run the compiler tests with:

```bash
cargo test
```

For browser execution, serve the runtime directory over HTTP and put the generated `out.wasm` beside `runtime/index.html`.

## Roadmap

- [x] Compiler executable
- [x] WebAssembly module generation
- [x] Lexer
- [x] Parser and AST
- [x] Arithmetic expressions
- [x] Variables
- [x] `if` / `elseif` / `while`
- [x] Functions
- [x] Strings and browser memory runtime
- [x] Booleans and logical operators
- [x] Loop `break`
- [ ] Static type checker
- [ ] Arrays
- [ ] Objects / tables
- [ ] Properties and methods
- [ ] Standard library
- [ ] Imports / modules
- [ ] Optimizations
- [ ] CLI diagnostics with source locations
