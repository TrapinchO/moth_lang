# COMPLETELY OUTDATED, NO GUARANTEES OF STUFF SHOWN HERE WORKING

# Basic data types
## Integers
Integers are numbers. Simple. Internally they are 32-bit signed ints (`i32`).

## Floats
Floating point numbers are internally 32-bit (`f32`). Unlike many languages, both whole and decimal parts must be present.
```rs
0.0; 1.0; 0.1; 1.1; // valid
.1; 1.; .; // invalid
```

## Booleans
Like many other languages, the boolean values are `true` and `false`. Note that unlike languages like Python they cannot be handled like numbers.

## Strings
Strings are delmimited by double quotes (`"`). They cannot be multiline, not terminating on newline throws an error.

Following escape characters are supported: `\n`, `\t`, `\"`, `\'` and `\\`.

```rs
"Hello world!"; // valid
"He\tllo\n \"world\"!"; // also valid
"Hello \a world!"; // invalid espace character
"Hello ; // invalid, reached EOL
```

## Characters
To be implented.

## Lists
Like Python, lists are made using square brackets. Trailing comma is supported.
```rs
[]; [1, 2, 3]; // valid
[1, "hi", true, [1, 2], ()]; // also valid
[1, 2,] // trailing comma, valid
```

Lists are indexed using square brackets. Valid indexes range from 0 to the length of the list exclusive. Negative indexes from minus length of the list to -1 can be used for backwards indexing.
```rs
let x = [1, 2, 3, 4, 5];
x[0]; // valid - gets 1
x[5]; // invalid - out of range
x[-5]; // valid - gets 1
x[-1]; // valid - gets 5
x[1] = 100; // sets 2nd value to 100
```

NOTE: until proper types are implemented, the lists do not have to be homogeneous.

## Unit
The unit type has a single value, `()`. Just like Rust, it is used when there is no meaningful value to be used. Functions by default return unit.

# Operators:
## Unary
Mothlang supports the numeric negation `-` and logic negation `!` operators.
```rs
-1; // valid
- - 1; // also valid, becomes 1
--1; // invalid - "--" is parsed as a single operator
!true; // valid
!!true // invalid, see above
```

## Binary
Mothlang supports custom binary operators with custom precendence. See `function` section for more information.

The current builtin operators are `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `>`, `>=`, `<`, `<=`, `&&`, `||`. `+` operator also supports string concatenation.

NOTE: integers and floats cannot be mixed and they return their respective type, i.e. `1 + 1.0` throws an error and `1 / 4` returns `0` (just like Rust).


# Variables
Variables are declared using `let` keyword. A variable cannot be overshadowed, i.e. a variable of the same cannot be redeclared. This applies to different scopes as well.

Valid variable names are strings composed of any letter (even with diacritics) or an underscore.
```rs
_; _test; te_st; test_; t3st;  // all valid names

let x = 10;
let x = 100;  // invalid - variable x is already declared
{
    let x = 10; // invalid 
}
```

Variables can be reassigned.
```rs
let x = 10;
x;  // returns 10
x = 1000;
x;  // returns 1000
```

NOTE: variables will explicitly have to be declared mutable in the future.


# Comments
Line comments are made with double forward slash `//`. Multiline comments are made by surrounding the comment by `/*` and `*/`.
<br>NOTE: line comment must not be immediately followed by a symbol, as that is parsed as a single operator
<br>NOTE: like line comments, multiline comment beginning must contain only the leading slash and stars
<br>NOTE: the ending can contain any number of leading starts and must be terminated with a slash
<br>NOTE: as of now, the comments cannot be nested
```rs
1+1;  // this comment is ignored
1+1;  // - this is fine, there is a space
1+1;  //- invalid - //- is an operator
1+1;  //// valid, as it is a slash
/*
and this comment describes in great detail why I kept the expression
above in the code despite it not doing absolutely anything
expect taking space and making the compiler and clippy angry at me.
*/
/* this is valid */
/** this is also valid - only stars ****/
/*= ivalid - an operator */
/* also invalid */*
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
} else if false {
    print("this will never be executed");
    // but will raise compiler warning
} else {
    print("x is either ten or eleven");
}
```

## While
While is made with `while`, a condition (again, without parentheses) and a block of statements to repeat
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
Code can be surrounded in braces `{}` to form a block. This can be useful for temporary variables.
```rs
{
    print("this is a nice code block, isnt it?");
    let x = 10;
}
x; // invalid, this no longer exists
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
}
```

Functions are called just like in C-style languages.
```
print(test(4, 2, false));  // prints 6, returns 2
print(test(1, 2, true));  // prints 3, returns unit
```

## Operator functions
Custom operators are declared just like regular functions, except with a symbol instead of name. Valid symbol characters are `+ - * / = < > ! | . $ & @ # ? ~ ^ : %`. Symbols cannot be `=` (assignment), `.` (field access), `?` (reserved) and `|` (lambda declaration). Symbols consisting of leading stars and ending with a slash (e.g. `*/` or `*****/`) are also prohibited to avoid confusion with block comment end.

Similarly to Haskell operators can have a custom precedence and associativity. Higher precedence means it will be evaluated sooner. For reference, addition (`+`) has associativity `5` and multiplication (`*`) `6`. Its values can range between 0 and 10 inclusive. Associativity is marked by the `infixl` and `infixr` keywords respectively. Most operators (like aforementioned addition) are left-associative. Defaults are precedence 0 and left associativity.
```rs
// valid
fun ++(a, b) {
    return a + a + b + b;
}

// also valid
infixl 0
fun <<(f, g) {
    fun _(x) {
        f(g(x));
    }
    return _;
}

// invalid - precedence out of range
infixr 100
fun --(x) {
    return x - 1;
}

// invalid -  resembles block comment end
fun ****/() {}
```

## Lambda functions
Anonymous functions are defined just like in Rust, that is parameters separated by `|` and then follow either by a single expression or a block. They behave just like regular functions.
```rs
|x, y| x + y; // valid
|| print(0); // valid - no parameters
|x, y| { print(x + y); } // valid - body
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

NOTE: At this point in time the validity of fields is NOT enforced during COMPILATION due to the lack of a type system.


## Impls
All methods for a given struct are in an extra `impl` block. Methods access the struct's data through the first parameter (usually named `self`) and may mutate it at will. They can be called through the field access syntax.

```rs
struct Point { x, y }
impl Point {
    fun manhattan(self) {
        return self.x + self.y;
    }
    fun scale(self, n) {
        self.x = self.x * n;
        self.y = self.y * n;
    }
    fun static(x, y) { /* ... */ } // doesnt work - `x` would become the new `self`
}


let p = Point(1, 2);
print(p.manhattan()); // prints 3
```
NOTE: static methods are not implemented.

NOTE: operator functions are not supported.

