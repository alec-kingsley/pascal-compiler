
# Minimal pascal compiler to x86-64 assembly (AT&T syntax). 

## Supported features

1. Types
	1. integer, boolean, char, real, string, text, array (must be in format: array[start..end] of type)
	2. built-in type conversion: `ORD()`, `CHR()`
2. Logic blocks
	1. if, if-else, for, while, repeat-until, begin-end
3. Literals
    1. characters and strings both enclosed in ''
    2. minimal support for scientific notation (example: epsilon := 1E-6)
4. Constants
    1. `CONST` block as well as some pre-defined constants like `maxint`, `true`, and `false`
5. Variables
6. Math
	1. all common operators
	2. built-in functions: `SQR()`, `SQRT()`
7. Built-in procedures
	1. `READ()`, `READLN()`, `WRITE()`, `WRITELN()`

## Design choices

1. Semicolons separate statements rather than end them
2. Does not automatically initialize variables

## Running

Compile with:
```
cargo run -- program.pas program.s
gcc program.s -o program -lm
```

## Known issues

1. String input limited to 255 bytes
2. Doesn't sanitize strings, currently throws them into printf (ignored since I don't want to keep libc dependency)
3. Blatant syntax error reporting seems pretty solid but need a nice looking warning/error reporting for other errors
4. Error reporting for unrecognized identifiers just crashes program
5. Calloc's every time a string is read in and free's none of it (after dropping libc dependency I'll store strings a different way entirely)
6. Support for special characters like ≥ is poor and they should be avoided for now

## Next planned features

1. Expand scientific notation to allow for decimal before E
2. Add more common math functions (ln, exp)
3. Implement number formatting using colons in write calls
4. Add procedures
5. Add functions


