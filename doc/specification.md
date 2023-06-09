# Data types
## Integers
Integers are numbers. Simple. Internally they are 32-bit signed ints (`i32`).

## Floats
Floating point numbers are internally 32-bit (`f32`). Unlike many languages, both whole and decimal parts must be present.
```rs
// valid
0.0; 1.0; 0.1; 1.1;
// invalid
.1; 1.;

```

## Booleans
Like many other languages, the boolean values are `true` and `false`. Note that unlike languages like Python they cannot be handled like numbers.

## Strings
Strings are delmimited by double quotes (`"`). They cannot be multiline, not terminating on newline throws an error.

Mothlang does not have a char type (yet).

## Other types
No other types are implemented so far.

## Examples
```rs
1; 20; 42;  // int
1.0; 0.0001;  // float
true; false;  // bool
"hello world!";  // string
"hello world!
"; // error! string not terminated before newline
```

# Operators:
Mothlang supports custom operators with custom precendence for both unary and binary. (TBI)

The current operators operating on the data types are `+`, `-` (both binary and unary), `*`, `/`, `%`, `==`, `!=`, `>`, `>=`, `<`, `<=`, `!` (unary), `&&`, `||`. `+` operator also supports string concatenation.
NOTE: integers and floats cannot be mixed and they return their respective type, i.e. `1 + 1.0` throws an error and `1 / 4` returns `0` (just like Rust).


# Variables
Variables are declared using `let` keyword. A variable cannot be overshadowed, i.e. a variable of the same cannot be declared in the current scope.

Valid variable names are strings composed of any letter (even with diacritics) or an underscore.
```rs
_; _test; te_st; test_;  // all valid names

let x = 10;
let x = 100;  // error! variable x is already declared
```
Variables can be reassigned. (NOTE: this will be changed in the future)
```rs
let x = 10;
x;  // returns 10
x = 1000;
x;  // returns 1000
```

# Comments
Line comments are made with double forward slash `//`. Multiline comments are made by surrounding the comment by `/*` and `*/`.
<br>NOTE: line comment must not be immediately followed by a symbol, as that counts as an operator
<BR>NOTE: like line comments, multiline comment beginning must contain only the leading slash and stars
<br>NOTE: as of now, the comments cannot be nested
```rs
1+1;  // this comment is ignored
1+1;  // - this is fine, there is a space
1+1;  //- error! //- is an operator
1+1;  //// this is fine, as it is a slash
/*
and this comment describes in great detail why I kept the expression
above in the code despite it not doing absolutely anything
expect taking space and making the compiler and clippy angry at me.
*/
/* this is ok */
/** this is also ok, only stars */
/*= not ok, an operator */
/* also not ok */*
```


# Control flow
## If/else
If statement is made using the keyword `if`, the condition (without parentheses, unlike Java and similar languages) and a block of statements surrounded by braces.
```rs
if 1 == 1 {
    let x = 10;
    x;  // note that there is no print function yet, and this all expression statements are printed
}
```
It can be also directly followed by `else`, which will be executed if the condition is false.
```rs

if 2 == 1 {
    let x = "true";
    x;
} else {
    "false"
}
```
NOTE: a block must follow if/else, unlike languages like C or Java
<br>NOTE: `else if` is not yet implemented

## While
While is made with `while` keyword (again, without parenthese) and a block of statements to repeat
```rs
while true {
    "Hello!";
}
```


# Blocks
Code can be surrounded in braces `{}` to form a block. This can be useful to have only temporary variables.
NOTE: not implemented yet
```rs
{
    "this is a nice code block, isnt it?";
    let x = 10;
}
x;  // error, this no longer exists
```