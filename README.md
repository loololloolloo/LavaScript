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

The compiler bootstrap supports:

- integer numbers and arithmetic: `+`, `-`, `*`, `/`, `%`
- numeric separators: `1_000`
- hexadecimal and binary integers: `0xff`, `0b1010`
- variables with `let` or `local` and reassignment
- immutable `const` variables
- booleans: `true`, `false`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- logical operators: `and`, `or`, `not`
- JavaScript-style logical spellings: `&&`, `||`, `!`
- `if`, `elseif`, `else`, and `end`
- `while` loops
- numeric `for` loops with positive or negative literal steps
- `repeat ... until` loops
- `do ... end` blocks
- `break` and `continue` in `while` / `for` loops
- functions with parameters and return values
- string literals using single or double quotes
- string escapes: `\\n`, `\\r`, `\\t`, `\\"`, `\\'`, and `\\\\`
- `print` for integers, booleans, and string literals
- `--` and `//` single-line comments
- semicolons as optional statement separators
- WebAssembly linear memory for string data
- browser execution through the included runtime

Example:

```lavascript
const start = 1

for i = start, 8
    if i == 3 then
        continue
    end
    if i == 7 then
        break
    end
    print i
end

let n = 0
repeat
    n = n + 1
until n >= 3

print 'done'
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
- [x] Extended loop control
- [x] Constants and numeric literal formats
- [ ] Static type checker
- [ ] Arrays
- [ ] Objects / tables
- [ ] Properties and methods
- [ ] Standard library
- [ ] Imports / modules
- [ ] Optimizations
- [ ] CLI diagnostics with source locations
