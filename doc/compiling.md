# Lexing
The source code string is separated into tokens for the parser to process.

Comments are removed during lexing.

# Parsing
The list of tokens is changed into the ast.

At this point binary operator precedence is ignored and unconditionally right associative.

# Reassociating
The binary operators are reassociated according to their respective precedence and associativity.

Also serves as a validation for operator existence.

# Variable checking
Variables and functions are checked for being declared exactly once. Using undeclared variables or redeclaring throws an error.

During this phase warnings for unused variables  are emitted.

# Simplifying
The AST is simplified into minimal instruction set. Notably, all operators are changed into function calls (including unary operators) and function declarations are changed into lambda expressions.

Also features basic constant folding for integers and floats.

# Interpreting
