
# Minimal pascal compiler to x86-64 assembly (AT&T syntax). 

## Supported features

1. Types
	1. integer, boolean, char, real, string, text, array (must be in format: array[start..end] of type)
	2. built-in type conversion: `ORD()`, `CHR()`
2. Logic blocks
	1. if, if-else, for, while, repeat-until, begin-end
3. Constants
    1. `CONST` block as well as some pre-defined constants like `maxint`, `true`, and `false`
4. Variables
5. Math
	1. All common operators
	2. built-in functions: `SQR()`, `SQRT()`
6. Built-in procedures
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
2. Doesn't sanitize strings... Back slashes should be preceded by a back slash, as well as % maybe
3. Blatant syntax error reporting seems pretty solid but need a nice looking warning/error reporting for other errors
4. Error reporting for unrecognized identifiers just crashes program
5. Calloc's every time a string is read in and free's none of it
6. Sometimes fails when special characters like ≥ are used

## Next planned features

1. Thoroughly evaluate signed float comparasion
2. Add functions / procedures


