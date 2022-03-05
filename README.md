# Oxy

## About

- A toy language implemented as a learning exercise and kinda code along for an amazing book [CraftingInterpreters](http://craftinginterpreters.com/)

- It is a sweet spot between javascript/java/lua/ruby/python being dynamic and garbage collected and semantic features from all

- It contains two interpreters:

  - A Tree-walk interpreter implemented in Rust (named roxy)
  - A Bytecode interpreter implemented in C (named coxy)

  (Ah yes I know I should have given more thought while naming it)

## Running the interpreters:

- Rust:

From the root of the project run:

```rust
cargo run -- <filename>
```

- C:

From the `bytecode-interpreter/` dir run:

```C
make compile
./coxy <filename>
```

## Lox Programs:

- Wanna explore the language?

  - Check the `examples/` dir

## Language Grammar:

- Language grammar can be found in `src/parser/parser.rs`
