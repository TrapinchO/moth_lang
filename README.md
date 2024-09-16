# Introduction
Mothlang is a learning project made to understand the inner workings of programming languages, and one day hopefully create an actually usable thing. It is also my first project in Rust.

The basic premise was to create a C-style language but further into the functional paradigm. It takes most inspiration from Rust and a bit from Haskell.

# Usage
When running the program without arguments it enters a repl mode, where a line of code is evaluated at a time. If ran with one argument, a file name, it evaluates the file instead.

Please note that the repl is very primitive and all code MUST be on a single line and no variable are preserved between inputs.

# Status
- [x] operator reassociation
- [x] variable checking
- [x] dead code checker
- [ ] optimizations
- [ ] std library
- features
  - [ ] type checking
  - [ ] enforced purity
  - [ ] proper mutability
  - [x] functions
  - [x] function definition
  - [x] lambdas
  - [x] lists
  - [x] structs
  - [x] impls
  - [ ] enums
  - [ ] modules


# Building and running
Make sure you have rust (and cargo) installed and run `cargo r <filename>` or `cargo r` for file and repl modes respectively.
