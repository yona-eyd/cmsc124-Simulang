# Extension

## Creators

- Adrienne Nicole V. Tipon (strg-ze)
- Ryona Cassandra P. Honrado (rcssndr)

## Overview

Simulang is a constraint-based language for people who would rather describe a system than script it. Instead of loops, a program declares entities (things that hold state) and laws (rules that act on them), then lets the system evolve until a stated condition holds. Writing it feels like setting up the rules of a small world and watching it settle, rather than stepping through instructions one at a time.

## Host language and build

- Host language: Rust 1.98.0
- Version metadata: [file that pins it, e.g. rust-toolchain.toml, go.mod]
- Build: `./build.sh`
- [Anything a fresh clone needs to know.]

## Running it


| Command | What it does |
|---|---|
| `./run <file>` | [Executes a program. Available from Lab 4.] |
| `./run --tokenize <file>` | [Prints the token stream.] |
| `./run --parse <file>` | [Prints the parsed tree.] |
| `./run --eval <file>` | [Evaluates each expression and prints its value.] |
| `./run` | [Starts the REPL.] |


Exit codes: 0 when file scans with no errors, 65 when scanner rejects an input, 70 on runtime errors.

## File extension

`.src` [Must match the `ext` field in every tests/lab*/manifest.json.]

## Lexical structure

### Keywords


| Keyword | Purpose |
| `entity` | Declares an entity, a named thing with its own  state |
| `law` | Declares a rule that acts on entities |
| `evolve` | Repeatedly applies laws; replaces loop |
| `until` | Gives the stopping condition of an `evolve` |
| `when` | Conditional(the language's if) |
| `var` | Declares a variable |
| `print` | Prints a value |
| `return` | Returns a value from a law |
| `and` | Logical conjuction |
| `or` | Logical disjunction |
| `true` | Boolean literal |
| `false` | Boolean literal |
| `nil` | The absence of a value |

### Operators


| Operator | Category |  Operands  | Associativity |             Precedence           |
|----------|----------|------------|---------------|----------------------------------|
|   `(`    | grouping |    none    |      none     | highest, overides all precedence |
|   `)`    | grouping |    none    |      none     | highest, overides all precedence |
|   `{`    |  block   |    none    |      none     |             none                 |
|   `}`    |  block   |    none    |      none     |             none                 |
|   `,`    |separator   |    none    |      none     |             none                 |
|   `:`    |separator |    none    |      none     |             none                 |
|   `;`    |separator |    none    |      none     |             none                 |
|   `->`    | arrow   |    none    |      none     |             none                 |
|   `.`    |separator |    none    |      none     |             none                 |
|   `+`,`-`|arithmetic|   binary   |      left     |             3                 |
|   `-`    |arithmetic|binary/unary|      none     |             none                 |
|   `*`    |arithmetic|   binary   |      none     |             none                 |
|   `/`    |arithmetic|    none    |      none     |             none                 |
| `!` | logical | unary | right | 1 |
| `-` (unary) | arithmetic | unary | right | 1 |
| `*` `/` `%` | arithmetic | binary | left | 2 |
| `+` `-` | arithmetic | binary | left | 3 |
| `<` `<=` `>` `>=` | comparison | binary | left | 4 |
| `==` `!=` | equality | binary | left | 5 |
| `and` | logical | binary | left | 6 |
| `or` | logical | binary | left | 7 |
| `=` | assignment | binary | right | 8 (lowest) |

### Literals


| Kind | Syntax | Produces |
|---|---|---|
| number | `42`, `3.14` | a 64-bit float (`f64`) |
| string | `"hello"` | a `String` holding the text between the quotes |
| boolean | `true`, `false` | keyword token; the token's literal is `null` |
| nil | `nil` | keyword token; the token's literal is `null` |


### Identifiers

- Start characters: `A-Z`, `a-z`, `_`
- Continue characters: `A-Z`, `a-z`, `0-9`, `_`
- Case-sensitive: yes (`var` is a keyword, `Var` is an identifier)
- Keywords are reserved and cannot be used as identifiers. A name is read in full before the keyword check, so `variable` is an identifier, not `var` followed by `iable`.
- ASCII only. Any non-ASCII character is a lexical error.

### Comments

- Line comments: `//`, reads until end of line
- Block comments: not supported
- Nesting: not supported
- Harness note: `comment_prefix` in `tests/lab*/manifest.json` is set to `//`

## Whitespace and termination

- Whitespace significant: no
- Statement terminator: semicolon (`;`)
- Block delimiters: braces (`{ }`)
- Grouping delimiters: parentheses (`( )`)

## Token output format

```
Token(type=VAR, lexeme=var, literal=null, line=1)
```

`type` is token category  (from Keywords/Operators/ Literals)
`lexeme` is raw source text
`literal` is the runtime value for literals only (`null if not applicable`)
`line` is 1-indexed

Frozen as of Lab 1; changes are recorded in the changelog.

## Grammar

```
program     → ( expression ";" )* EOF
expression  → assignment
assignment  → IDENTIFIER "=" assignment | logic_or
logic_or    → logic_and ( "or" logic_and )*
logic_and   → equality ( "and" equality )*
equality    → comparison ( ( "!=" | "==" ) comparison )*
comparison  → term ( ( ">" | ">=" | "<" | "<=" ) term )*
term        → factor ( ( "-" | "+" ) factor )*
factor      → unary ( ("/" | "*" | "%" ) unary )*
unary       → ( "!" | "-" ) unary | primary
primary     → NUMBER | STRING | "true" | "false" | "nil" | IDENTIFIER  | "(" expression ")"

```

- Grammar is unambiguous as each level delegates only to the next tighter one, so a level only receive finished subtrees from tighter levels.
- Every binary level is left-associative as it is a loop, assignment and unary are right-associative because they recurse on the righ
- and/or are words with their on two levels
- % sits right with * and /
- ; terminates each expression


## Parse output format

```
[one line of real --parse output, e.g. (+ 1.0 (* 2.0 3.0))]
```

- Groupings print as: `(group <inner>)`, e.g.  ``
- Numbers print as: a a number with a decimal point, e.g. `42` prints as `42.0` and `3.14` as `3.14`

## Semantics

### Values and types

[What runtime values exist, and how they are represented in the host
language.]

### Value printing

- Numbers: [e.g. 5 rather than 5.0]
- Nil: [spelling]
- Strings: [with or without quotes]

### Truthiness

[The complete rule. Which values are false in a condition; everything else is
true.]

### Operator semantics

- Arithmetic: [accepted operand types]
- `+` on strings: [concatenation, error, or coercion]
- Mixed types: [what happens]
- Comparison: [accepted operand types]
- Equality across types: [false, or an error]
- Division by zero: [value produced, or runtime error]

### Scope and bindings

- Redeclaration in the same scope: [allowed or an error]
- Uninitialized variable holds: [value]
- Shadowing: [behavior]
- Undefined name: [static error with exit 65, or runtime error with exit 70]

### Control flow and functions

- Logical operators return: [booleans, or the operand]
- Dangling else binds to: [which if]
- Closure capture of a loop variable: [per iteration, or shared]
- Function with no return statement produces: [value]
- Arity mismatch: [message and exit code]

## Native functions


| Name | Arguments | Returns | Notes |
|---|---|---|---|
| [name] | [count and types] | [type] | [caveats] |


## Errors and diagnostics

Message format:

```
[one real static error]
[one real runtime error]
```


| Failure | Exit code |
|---|---|
| [lexical error] | 65 |
| [syntax error] | 65 |
| [runtime error] | 70 |


## Testing conventions


| Folder | Activity | Mode | Flag |
|---|---|---|---|
| tests/lab1 | Scanner | sidecar | `--tokenize` |
| tests/lab2 | Parser | sidecar | `--parse` |
| tests/lab3 | Evaluator | inline | `--eval` |
| tests/lab4 | Context | inline | none |
| tests/lab5 | Functions | inline | none |


```
[specific tests]...
```

Run locally with:

```bash
curl -sSL https://raw.githubusercontent.com/WhiteLicorice/cmsc-124-harness/v1.1/run_tests.py -o run_tests.py
./build.sh
python3 run_tests.py tests/lab1
```

## Sample code

```
[a short program]
```

Output:

```
[its output]
```

## Design rationale

[Why the language is the way it is. Cover the choices that surprised you, the
features you cut, and the decisions you reversed. Specific reasons, not
approval of your own work.]

## Known limitations

1. String literals do not process escape sequences. They are taken literally, like any other character inside "".
2. Comments are line comments only. No block comments.
3. Empty input still emits a single Eof token at the very first line.
4. scan_tokens() collects error in the file before returning rather than stopping at the first one.
5. REPL scans one line at a time. A token split across multiple lines is read as undfinished and returns an error immediately.
6. AST formats number with Rust's `{:?}` for `f64`, which switches to exponent form for values greater than or equal to 1e16 or less than 1e-4. (e.g. `10000000000000000;` prints `1e16`, `0.00001;` prints `1e-5`). Values in between print normally. 

## Changelog


| Activity | What changed in the language |
|---|---|
| Lab 1 | Defined the token set: punctuation, arithmetic, comparison and equality operators, `->`, keywords (`entity`, `law`, `evolve`, `until`, `when`, `var`, `print`, `return`, `and`, `or`, `true`, `false`, `nil`), number and string literals, and identifiers |