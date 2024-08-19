# Intro
Mothlang is a project made to understand the inner workings of programming languages, and one day hopefully create an actually usable thing.

# Usage
When running the program without arguments it enters a repl mode, where a line of code is evaluated at a time. If ran with one argument, a file name, it evaluates the file instead.

Please note that the repl is very primitive and all code MUST be on a single line.

# Status
- [x] operator reassociation
- [x] variable checking
- [x] dead code checker
- [ ] optimizations
- [ ] std library
- features
  - [ ] type checking
  - [x] functions
  - [x] function definition
  - [x] lambdas
  - [x] lists
  - [x] structs
  - [ ] impls
  - [ ] enums
  - [ ] modules


# Building and running
Make sure you have rust (and cargo) installed and run `cargo r <filename>` or `cargo r` for file and repl modes respectively.
(NOTE: currently no variables or functions can be preserved between repl inputs)
