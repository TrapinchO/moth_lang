# Lexing
The source code string is separated into tokens for the parser to process.

Comments are removed during lexing.

# Parsing
The list of tokens is changed into the ast.

At this point binary operator precedence is ignored and unconditionally right associative.

# Reassociating
The binary operators are reassociated according to their respective precedence and associativity.

Also serves as a validation for operator existance.

# Variable checking
Variables and functions are checked for being declared exactly once. Using undeclared variables or redeclaring throws an error.

TBA: checking for unused variables/names.

# Interpreting
