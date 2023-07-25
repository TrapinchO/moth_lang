# moth_lang
Mothlang is a project made to understand the inner workings of programming languages, and one day hopefully a usable thing.

# usage
When running the program without arguments it enters a repl mode, where a line of code is evaluated at a time. If ran with one argument, a file name, it evaluates the file instead.

# status
- [x] lexer
- [ ] parser
  - [x] expressions
  - [ ] statements
- [x] reassociation
- [ ] checks
  - [ ] types
  - [ ] other stuff
- [x] interpreter (provisional)
- [x] errors (framework is in place)
  - [ ] error types
- [ ] tests
  - [x] lexer
  - [ ] parser
    - [x] expressions (just a few)
    - [ ] statements
  - [ ] reassociation
  - [ ] interpreter
  - [ ] errors


# Building and running
Make sure you have rust (and cargo) installed and run `cargo r <filename>` or `cargo r` for file and repl modes respectively.
