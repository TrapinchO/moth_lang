# COMPLETELY OUTDATED, NO GUARANTEES OF STUFF SHOWN HERE WORKING

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

Following escape characters are supported: `\n`, `\t`, `\"`, `\'` and `\\`.

Mothlang does not have a char type (yet).

## Lists
```rs
[]; [1, 2, 3];  // valid
[1, "hi", true, [1, 2], ()];  // also valid
[1, 2,]  // invalid - trailing comma
```
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
[1, 2, 3];
```

# Operators:
## Unary
Mothlang supports the numeric negation operator `-` and logic negation `!`.
```rs
-1;
- - 1;
--1;  // error - "--" is interpeted as a single operator
!true;
!!true // error, see above
```

## Binary
Mothlang supports custom binary operators with custom precendence. (TBI)

The current operators are `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `>`, `>=`, `<`, `<=`, `&&`, `||`.

`+` operator also supports string concatenation.

Custom operators are declared just like regular functions. Similarly to Haskell a custom precedence and associativity. Higher precedence means it will be evaluated sooner. For reference, addition (`+`) has associativity `5` and multiplication (`*`) `6`. Its values can range between 0 and 10 inclusive. Associativity is marked by the `infixl` and `infixr` keywords respectively. Most operators (like aforementioned addition) are left-associative.
```rs
infixl 0
fun <<(f, g) {
    fun _(x) {
        f(g(x));
    }
    return _;
}
```

NOTE: integers and floats cannot be mixed and they return their respective type, i.e. `1 + 1.0` throws an error and `1 / 4` returns `0` (just like Rust).

NOTE: an operator composed of only leading stars and an ending slash (e.g. `*/` or `****/`) is not a valid operator to prevent confusion

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

Variables cannot be redeclared (shadowed), even in different scopes.
```rs
let x = 10;
let x = 10;  // error - already declared variable
{
    let x = 10; // error
}
```

# Comments
Line comments are made with double forward slash `//`. Multiline comments are made by surrounding the comment by `/*` and `*/`.
<br>NOTE: line comment must not be immediately followed by a symbol, as that counts as an operator
<br>NOTE: like line comments, multiline comment beginning must contain only the leading slash and stars
<br>NOTE: the ending can contain any number of leading starts and must be terminated but a slash
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
/** this is also ok, only stars ****/
/*= not ok, an operator */
/* also not ok */*
```


# Control flow
## If/else
If statement is made using the keyword `if`, the condition (without parentheses, unlike Java and similar languages) and a block of statements surrounded by braces.
```rs
if 1 == 1 {
    let x = 10;
    print(x);
}
```
It can be also directly followed by `else`, which will be executed if the condition is false.
```rs

if 2 == 1 {
    let x = "true";
    print(x);
} else {
    print("false");
}
```
NOTE: a block must follow if/else, unlike languages like C or Java

Multiple conditions can be chained with `else if`. When one of the conditions is true, the rest is not checked. `else` can go after the chain like usual.
```rs
let x = 10;
if x == 0 {
    print("x is zero");
} else if x < 10 {
    print("x is smaller than ten");
} else if x > 11 {
    print("x is greater than eleven");
} else {
    print("x is either ten or eleven");
}
```

## While
While is made with `while` keyword (again, without parenthese) and a block of statements to repeat
```rs
while true {
    print("Hello!");
}
```
Continue and break statements are supported.
```rs
let i = 0;
while true {
    i = i + 1;
    if i % 10 == 0 {
        continue;
    }
    if i >= 100 {
        break;
    }
    print(i);
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

# Functions
Functions are defined with the `fun` keyword, followed by the function name, a list of parameters and the body. If the function returns without the `return` statement or no value is provided, `unit` is returned.
```kt
fun test(a, b, s) {
    print(a + b);
    if s == true {
        return;
    }
    return a / b;
```

Functions are called just like the C-style languages.
```
print(test(4, 2, false));  // prints 6, returns 2
print(test(1, 2, true));  // prints 3, returns unit
```

# Structs
NOTE: This is a subject to change once proper type system is implemented. It will be almost surely same as Rust.

Struct are defined using the `struct` keyword, followed by the struct name and a list of fields.

```rs
struct Point {
    x,
    y,
}
```

Structs are instantiated just like Python or Kotlin classes (i.e. a regular function call, no `new` keyword or special syntax). Fields are accessed using the dot notation. Additional fields may not be created at runtime.
```rs
let p = Point(1, 2);
print(p.x, p.y);
p.x = 10;
p.z = true;  // ERROR: Field \"z\" does not exist
```

NOTE: Unlike dynamic languages
NOTE: The validity of fields is NOT enforced during COMPILATION due to the lack of a type system.


