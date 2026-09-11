# LavaScript Language Reference

LavaScript is compiled directly to WebAssembly. Its syntax intentionally mixes Lua-like block structure with JavaScript-like operators while keeping its own compiler and runtime model.

## Core syntax

```lavascript
const limit = 10
let total = 0

for i = 1, limit
    total = total + i
end

print total
```

## Values

Currently supported scalar values:

- integer numbers (`42`, `1_000`, `0xff`, `0b1010`)
- booleans (`true`, `false`)
- strings (`"hello"`, `'hello'`)

WebAssembly currently represents numeric and boolean expressions as `i32`. Strings are stored in linear memory and passed to the browser runtime as pointer/length pairs for printing.

## Operators

Arithmetic: `+`, `-`, `*`, `/`, `%`

Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`

Logic: `and`, `or`, `not`, `&&`, `||`, `!`

## Control flow

- `if ... elseif ... else ... end`
- `while ... end`
- `for name = start, end[, step] ... end`
- `repeat ... until condition`
- `do ... end`
- `break`
- `continue`

## Functions

```lavascript
function square(x)
    return x * x
end

print square(12)
```

Functions currently use integer/boolean-compatible WebAssembly parameters and an `i32` return ABI.

## Comments

Both Lua-style and JavaScript-style single-line comments are accepted:

```lavascript
-- Lua-style
// JavaScript-style
```

## Compiler architecture

```text
source -> lexer -> parser/AST -> semantic checks -> Wasm codegen -> .wasm -> host runtime
```

The implementation is intentionally being built in layers. Collection syntax tokens are reserved for the upcoming array/table implementation; arrays and objects are not yet runtime values.

## ABI

The browser runtime supplies:

- `lavascript.print_i32(i32)`
- `lavascript.print_string(i32 pointer, i32 length)`

The generated module exports:

- `main()`
- `memory`

This ABI keeps the compiler independent from the browser while allowing the same Wasm output to be hosted by another JavaScript environment later.

## Roadmap

1. Static scalar type checking
2. Arrays and indexed access
3. Tables/objects and property access
4. Methods and closures
5. Built-in math/string functions
6. Module imports/exports
7. Source spans and compiler diagnostics
8. Constant folding and dead-code elimination
9. Optimized Wasm generation
10. Package/distribution tooling
