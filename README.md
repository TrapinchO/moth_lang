# moth_lang
Mothlang is a project made to understand the inner workings of programming languages, and one day hopefully a usable thing.

# usage
When running the program without arguments it enters a repl mode, where a line of code is evaluated at a time. If ran with one argument, a file name, it evaluates the file instead.

# status
- [x] operator reassociation
- [x] variable checking
- [ ] dead code checker
- [ ] optimizations
- features
  - [ ] type checking
  - [x] functions
  - [x] function definition
  - [x] lambdas
  - [x] lists
  - [x] structs
  - [ ] impls
  - [ ] enums


# Building and running
Make sure you have rust (and cargo) installed and run `cargo r <filename>` or `cargo r` for file and repl modes respectively.
(NOTE: currently no vaariables or functions can be preserved between repl inputs)
