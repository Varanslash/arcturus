# ARCTURUS Toolchain Documentation

## Table of Contents

1. [Overview](#overview)
2. [Toolchain Components](#toolchain-components)
3. [The ARC Language](#the-arc-language)
   - [Program Structure](#program-structure)
   - [Preprocessor Directives](#preprocessor-directives)
   - [Comments](#comments)
   - [Types and Literals](#types-and-literals)
   - [Keywords and Instructions](#keywords-and-instructions)
   - [Variables](#variables)
   - [Arithmetic](#arithmetic)
   - [Control Flow](#control-flow)
   - [Conditional Branching](#conditional-branching)
   - [Functions](#functions)
   - [I/O](#io)
4. [hydrae — The Compiler](#hydrae--the-compiler)
   - [Usage](#hydrae-usage)
   - [Compilation Pipeline](#compilation-pipeline)
   - [Compiler Flags](#compiler-flags)
   - [Output Modes](#output-modes)
5. [muphrid — The Virtual Machine](#muphrid--the-virtual-machine)
   - [Usage](#muphrid-usage)
   - [VM Architecture](#vm-architecture)
   - [Execution Model](#execution-model)
6. [spica — The Assembler/Disassembler](#spica--the-assemblerdisassembler)
   - [Usage](#spica-usage)
   - [Assembling](#assembling)
   - [Disassembling](#disassembling)
7. [AVM Bytecode Reference](#avm-bytecode-reference)
   - [Bytecode Format](#bytecode-format)
   - [Full Instruction Set](#full-instruction-set)
8. [AVM Assembly Reference](#avm-assembly-reference)
9. [Language Quirks and Gotchas](#language-quirks-and-gotchas)
10. [Error Reference](#error-reference)
11. [End-to-End Examples](#end-to-end-examples)

---

## Overview

ARCTURUS is a small, stack-based programming language and virtual machine toolchain. It consists of three programs named after stars in the Boötes constellation and its neighbors:

- **hydrae** — compiles `.arc` source files into AVM bytecode (or x86-64 assembly)
- **muphrid** — a virtual machine that executes AVM bytecode
- **spica** — a standalone assembler and disassembler for the AVM text assembly format

The general workflow is:

```
source.arc  →  [hydrae]  →  program.avm  →  [muphrid]  →  execution
                   ↓
              program.asm  (optional x86-64 output)
```

Alternatively, if you want to work at the assembly level directly:

```
source.s   →  [spica assemble]    →  program.avm  →  [muphrid]
program.avm →  [spica disassemble] →  source.s
```

---

## Toolchain Components

| Binary | Source File | Role |
|--------|-------------|------|
| `hydrae` | `hydrae.rs` | High-level `.arc` compiler |
| `muphrid` | `muphrid.rs` | AVM bytecode interpreter |
| `spica` | `spica.rs` | AVM assembler / disassembler |

All three are written in Rust and can be compiled with `cargo build` or `rustc` individually.

---

## The ARC Language

ARC is a simple, imperative, stack-oriented language. It is low-level by design — closer to assembly than to a high-level language. There are no functions as first-class values, no loops as syntax, and no implicit scoping. Loops and conditionals are built from labels and conditional jumps.

### Program Structure

Every ARC program is wrapped in a pair of curly braces. These map to the `ARC_START` and `ARC_END` structural markers in the bytecode and are mandatory.

```arc
{
    # your code here
}
```

Multiple blocks can be chained using `}{` as a delimiter (internally emits `ARC_DELIM`). This is used to separate logical sections, e.g., a main block followed by function sub-blocks.

```arc
{
    call myFunc;
}{
    label myFunc;
    print "hello";
    return;
}
```

The program always terminates when an `exit` statement or the closing `}` of the outermost block is reached. The compiler automatically appends an exit block (`HYD_EXITBLOCK`) to handle normal program termination.

### Preprocessor Directives

Preprocessor directives begin with `%` and are processed before lexing. There are two directives:

**`%scope <filename>`**

Includes the contents of another `.arc` file inline. This happens textually — the file is read and appended to the source before compilation. Think of it like a `#include`.

```arc
%scope stdlib.arc
```

**`%define <n> as <address>`**

Defines a named memory address alias for use with x86-64 output mode (`-x`). After this directive, any identifier with the name `<n>` will be substituted with `<address>` during lexing. Addresses are written in hexadecimal using an `x` prefix instead of the usual `0x` — so `0x1000` is written as `x1000`.

```arc
%define BUFFER as x1000
%define RESULT as xA000
```

This directive is intended for x86-64 mode where you need to reference specific memory addresses. It has no meaningful effect in AVM bytecode mode since the AVM has no concept of raw addresses.

> **Note:** `%define` substitution only works on identifiers, not keywords, literals, or punctuation. The `as` keyword is required between the name and the address — omitting it is a hard `SyntaxError`.

### Comments

Comments begin with `#` and run to the end of the current block (until the next `#`). The lexer enters an `Inactive` state upon seeing `#` and resumes only when it encounters a `#`. This means `#` does not just comment out a single line — it comments out everything up to the next hash.

```arc
{
    # This is a comment. Everything here is ignored until the next #
    print "hello";   # This is also ignored — everything after a hash on any line #
}
```

### Types and Literals

ARC supports four value types:

| Type | Example Literals | Notes |
|------|-----------------|-------|
| Integer | `42`, `-7`, `0` | Stored as `i64` (signed 64-bit) |
| Float | `3.14`, `0.5` | Stored as `f64` (64-bit double) |
| String | `"hello"`, `"world"` | UTF-8, enclosed in double quotes |
| Boolean | `true`, `false` | Case-sensitive |

All values live on the evaluation stack. Variables are a named store backed by a `HashMap<String, StackType>`.

### Keywords and Instructions

The reserved keywords are: `label`, `exit`, `jump`, `return`, `call`, `jumpif`, `callif`, `let`, `print`, `input`.

The literals `true` and `false` are also reserved.

### Variables

Variables are declared and assigned with `let`. The syntax is:

```arc
let <name> = <expression>;
```

The expression is evaluated left-to-right and the result is stored in the named variable. Variables do not need to be declared before use in conditional expressions — but reading an undeclared variable at runtime will panic.

**Simple assignment:**

```arc
let x = 42;
let name = "Alice";
let flag = true;
let pi = 3.14;
```

**Arithmetic assignment:**

The right-hand side supports a chain of binary arithmetic operations. The left-most operand is pushed first, then each operator-operand pair is applied in sequence.

```arc
let x = 10 + 5;         # x = 15
let y = x - 3;          # y = (value of x) - 3
let z = 2 * 6 + 1;      # z = (2 * 6) + 1 = 13
```

The supported operators in `let` are `+`, `-`, `*`, `/`, `%`.

> **Quirk:** The `let` parser is left-to-right with no operator precedence. `2 + 3 * 4` evaluates as `(2 + 3) * 4 = 20`, not `2 + (3 * 4) = 14`. Use intermediate variables to control evaluation order.

**Loading a variable in an expression:**

When an identifier appears on the right-hand side of `let`, it is treated as a variable load (`LOAD`).

```arc
let a = 5;
let b = a + 3;   # b = 8
```

### Arithmetic

The arithmetic operations `+`, `-`, `*`, `/`, `%` are only available inside `let` expressions. There is no standalone arithmetic instruction in ARC source — arithmetic is always part of an assignment.

All arithmetic at the bytecode level operates on integers. Floating point values can be stored and loaded but the arithmetic instructions (`ADD`, `SUB`, `MUL`, `DIV`, `MOD`) in the AVM only support `Int` stack values; mixing types will panic the VM.

Integer division uses truncating division (like Rust's `/` on integers). The `%` operator returns the remainder.

### Control Flow

**Labels** mark positions in the code that can be jumped to:

```arc
label myLabel;
```

A label can appear anywhere inside a block. Labels are collected in a preprocessing pass by the VM before execution, so forward references work.

> **Naming convention:** Labels beginning with `HYD_` are reserved for the compiler's internal use. Do not define labels with this prefix in user code.

**Unconditional jump:**

```arc
jump myLabel;
```

Transfers execution to the named label immediately. There is no fall-through — this is an unconditional branch. Use this to implement loops:

```arc
{
    let i = 0;
    label loopStart;
    print i;
    let i = i + 1;
    jump loopStart;   # infinite loop
}
```

**Exit:**

```arc
exit;
```

Terminates the program immediately. Equivalent to jumping to the end of the program.

**Return:**

```arc
return;
```

Returns from a `call`. Should only be used inside a labeled function body. Using `return` outside a call context will pop a garbage value from the call stack and crash.

### Conditional Branching

**`jumpif`** and **`callif`** are the conditional forms of `jump` and `call`. The syntax is:

```arc
jumpif <label> <lhs> <operator> <rhs>;
callif <label> <lhs> <operator> <rhs>;
```

The condition is a comparison expression. Supported comparison operators:

| Operator | Meaning |
|----------|---------|
| `==` | Equal |
| `!=` | Not equal |
| `>` | Greater than |
| `>=` | Greater than or equal |
| `<` | Less than |
| `<=` | Less than or equal |

Multiple conditions can be chained with `&&` (AND), `||` (OR), or `^^` (XOR). These are collected and emitted after the comparisons:

```arc
jumpif myLabel x > 0 && x < 100;
```

**Implementing if-then-else:**

ARC has no `if/else` syntax. Use labels and conditional jumps:

```arc
{
    let x = 42;
    jumpif thenBlock x == 42;
    print "not 42";
    jump endIf;
    label thenBlock;
    print "it's 42";
    label endIf;
}
```

**Implementing a counted loop:**

```arc
{
    let i = 0;
    label loopTop;
    jumpif loopEnd i >= 10;
    print i;
    let i = i + 1;
    jump loopTop;
    label loopEnd;
    print "done";
}
```

### Functions

Functions are created by placing a labeled block of code and ending with `return;`. They are called with `call`:

```arc
{
    call greet;
    exit;
}{
    label greet;
    print "Hello!";
    return;
}
```

`call` pushes the return address onto the call stack. `return` pops it and jumps back. This gives ARC simple subroutine semantics. There is no argument-passing mechanism in the language itself — arguments must be set in variables before calling and read inside the function body.

```arc
{
    let x = 7;
    call double;
    print result;
}{
    label double;
    let result = x + x;
    return;
}
```

> **Important:** Variables are global. There is no stack frame for locals. All `let` assignments write to the same flat `HashMap`, so called functions can read and write the caller's variables freely.

### I/O

**Print:**

```arc
print <value>;
```

Prints a single value to stdout followed by a newline. The value can be a literal or a variable name:

```arc
print "hello";
print 42;
print myVar;
print true;
```

Only one value can be printed per `print` statement. To print multiple things, use multiple `print` statements or store a composed string in a variable first.

**Input:**

```arc
input <varname>;
```

Reads a line from stdin (stripping the trailing newline) and stores it as a string in the named variable.

```arc
input name;
print name;
```

> **Quirk:** `input` always produces a string value, regardless of what the user types. There is no built-in type coercion — if you need a number from input, you must handle it at a higher level, as ARC has no parsing/casting instructions.

---

## hydrae — The Compiler

**hydrae** is the primary compiler. It takes a `.arc` source file and produces either AVM bytecode (`.avm`) or x86-64 NASM assembly (`.asm`).

### hydrae Usage

```sh
./hydrae <input.arc> [flags]
```

The input file **must** have the `.arc` extension — the compiler enforces this. The output file is automatically named by replacing `.arc` with `.avm` (or `.asm` for x86-64 mode).

### Compilation Pipeline

hydrae processes source code through four sequential stages:

**Stage 1 — Preprocessing**

Two preprocessing passes run over the raw source text:

1. `preprocscope`: Scans for `%scope` directives and appends the contents of each referenced file to the source. Also unconditionally appends the compiler's exit block (`HYD_EXITBLOCK`) to the end of the source.

2. `preprocdefine`: Scans for `%define` directives and builds an identifier substitution table (`HashMap<String, String>`).

**Stage 2 — Lexing**

The preprocessed source is passed to the lexer, which produces a flat stream of `Token` values. The lexer is a character-by-character state machine with six states: `Idle`, `Inactive`, `Punct`, `Word`, `String`, and `Number`.

After lexing, all identifiers that match entries in the `%define` table are substituted with their defined replacements. A sentinel `HYD_EOF` identifier is appended to mark the end of the stream.

Tokens produced:
- `Punct(String)` — operators and delimiters
- `Keyword(String)` — reserved words
- `Identifier(String)` — user-defined names
- `Integer(i64)`, `Float(f64)`, `String(String)`, `Bool(bool)` — literals

**Stage 3 — Parsing**

The token stream is consumed by a recursive descent-style parser that emits a flat list of `Node` values (the AST). The parser is pointer-based, advancing through the token stream manually. There is no tree structure — the "AST" is a flat `Vec<Node>` because the language itself is linear.

Node types:
- `Start`, `End`, `Delim` — block delimiters
- `Label(String)`, `Jump(String)`, `Call(String)` — control flow
- `JumpIf(String)`, `CallIf(String)` — conditional control flow
- `Exit`, `Return` — termination
- `Let(String)` — variable assignment
- `Add(u8)`, `Sub(u8)`, `Mul(u8)`, `Div(u8)`, `Mod(u8)` — arithmetic
- `Compare(u8)` — comparison (opcode embedded as the byte)
- `Print(Box<Node>)` — output (the inner node is the value expression)
- `Input(String)` — input to variable
- `PushInt(i64)`, `PushFloat(f64)`, `PushString(String)`, `PushBool(bool)` — literals
- `Load(String)` — variable load
- `Logic(String)` — logical operator (`and`, `or`, `xor`)

**Stage 4a — Serialization (to AVM assembly text)**

The node list is serialized to a text-based AVM assembly format (a newline-separated list of mnemonic instructions). This is an intermediate representation — the actual bytecode or x86-64 assembly is produced from this text.

**Stage 4b — Assembly (to bytecode or x86-64)**

From the AVM assembly text, either:
- `native_assemble()` produces binary AVM bytecode (`Vec<u8>`) — the default
- `x86_64_assemble()` produces NASM-syntax x86-64 assembly text

### Compiler Flags

| Flag | Long form | Description |
|------|-----------|-------------|
| `-d` | `--debug` | Print detailed debug output at every stage |
| `-l` | `--lex` | Stop after lexing; output the token stream |
| `-p` | `--parse` | Stop after parsing; output the AST |
| `-s` | `--serialize` | Stop after serialization; output AVM assembly text |
| `-x` | `--x86_64` | Compile to x86-64 NASM assembly instead of AVM bytecode |
| `-h` | `--help` | Print help text and exit |

The stage-stop flags (`-l`, `-p`, `-s`) write their intermediate output to the `.avm` output file instead of bytecode, and also disable debug mode (these flags are mutually exclusive with `-d`).

### Output Modes

**Default (AVM bytecode):** Produces a `.avm` binary file suitable for execution with muphrid.

**x86-64 mode (`-x`):** Produces a `.asm` file in NASM syntax. This targets Linux x86-64 using raw system calls — `_start` is the entry point, and the program terminates with `hlt` + `jmp HYD_end`. Note that x86-64 output for `PRINT`, `INPUT`, and some other operations is not implemented in the x86-64 backend (those opcodes fall through silently), so x86-64 mode is best suited for arithmetic-heavy programs without I/O.

---

## muphrid — The Virtual Machine

**muphrid** executes AVM bytecode files produced by hydrae or spica.

### muphrid Usage

```sh
./muphrid <program.avm> [-debug]
```

The input file is a binary `.avm` file. The optional `-debug` flag enables verbose VM state output before each instruction.

### VM Architecture

muphrid is a register-free, stack-based interpreter. Its state consists of:

- **`stack: Vec<StackType>`** — the evaluation stack. All operations read from and write to this stack.
- **`callstack: Vec<i64>`** — the call stack. Stores return addresses for `call`/`return` pairs.
- **`data: HashMap<String, StackType>`** — the variable store. Named variables live here.
- **`labels: HashMap<String, usize>`** — the label table. Maps label names to byte offsets in the bytecode, built in a preprocessing pass before execution.
- **`ip: usize`** — the instruction pointer. Points to the current byte in the bytecode.

`StackType` represents the four value types: `Int(i64)`, `Float(f64)`, `String(String)`, `Bool(bool)`.

### Execution Model

**Preprocessing pass:** Before execution begins, muphrid scans the entire bytecode for `LABEL` instructions (`0xD4`) and records their positions in the label table. This allows forward jumps to work — a label doesn't need to appear before the jump that targets it.

**Validation:** The VM checks that the first byte is `0xF0` (ARC_START) and the last byte is `0xF1` (ARC_END). If either check fails, the program is rejected with a `SyntaxError`.

**Execution loop:** The main loop dispatches on `code[ip]` using a Rust `match`. Each instruction handler reads any operand bytes immediately following the opcode, performs its operation on the stack/data/callstack, and advances `ip` by the appropriate amount.

The program ends when:
- An `ARC_END` (`0xF1`) byte is encountered — the VM returns immediately.
- A `KernelError` panic is triggered by a type mismatch or invalid operation.

**Debug mode:** When started with `-debug`, the VM prints `ip`, the current opcode, the full stack, the data store, and the call stack before each instruction.

---

## spica — The Assembler/Disassembler

**spica** is a lower-level tool that works directly with the AVM text assembly format — it does not understand ARC source syntax. It has two modes: assemble and disassemble.

### spica Usage

```sh
spica assemble <source.s> <output.avm>
spica disassemble <program.avm> <output.s>
```

### Assembling

Reads a text file of AVM assembly mnemonics and produces a binary `.avm` file. This is useful for:
- Writing programs directly in AVM assembly without going through the ARC compiler
- Inspecting and re-assembling modified disassembly output

Each line of the input is one instruction. Lines are whitespace-split; the first token is the mnemonic and subsequent tokens are operands.

### Disassembling

Reads a binary `.avm` file and produces a human-readable text file of AVM assembly mnemonics. The output is a 1:1 translation of bytes back to their mnemonic representations.

This is useful for debugging, auditing, or understanding the output of the compiler. The disassembled output can be fed back into `spica assemble` to reproduce the original binary.

> **Note:** `spica disassemble` has a minor bug in the LOAD_STR, LOAD_BOOL, and LOAD_DEC handlers: they read the variable name starting at `ip + 1` instead of `ip + 2`, incorrectly including the length byte in the name. This means disassembly of programs with string/bool/float variable loads may produce garbled output for those variable names.

---

## AVM Bytecode Reference

### Bytecode Format

The AVM bytecode is a flat sequence of bytes. There is no header, no magic number, and no section table — the format is completely flat. Every valid program must begin with `0xF0` (ARC_START) and end with `0xF1` (ARC_END).

Instructions with operands encode their operands immediately following the opcode byte. String and label operands use a length-prefixed format: a single byte giving the UTF-8 byte length, followed by that many bytes of UTF-8 text. Integer values use little-endian 8-byte encoding. Floats use IEEE 754 little-endian 8-byte encoding. Booleans use a single byte (`0x01` = true, `0x00` = false).

### Full Instruction Set

#### Arithmetic (integers only)

| Opcode | Mnemonic | Operand | Description |
|--------|----------|---------|-------------|
| `0x01` | `ADD` | `u8 count` | Pop `count` integers, sum them, push result |
| `0x02` | `SUB` | `u8 count` | Pop `count` integers, subtract them, push result |
| `0x03` | `MUL` | `u8 count` | Pop `count` integers, multiply them, push result |
| `0x04` | `DIV` | `u8 count` | Pop `count` integers, divide them, push result |
| `0x05` | `MOD` | `u8 count` | Pop `count` integers, modulo them, push result |

#### Variable Operations

| Opcode | Mnemonic | Operand | Description |
|--------|----------|---------|-------------|
| `0x10` | `STORE` | `len, name` | Pop top of stack, store in variable `name` |
| `0x11` | `PUSH_INT` | `i64 (LE)` | Push 64-bit signed integer constant |
| `0x12` | `PUSH_STR` | `len, utf8` | Push string constant |
| `0x13` | `PUSH_BOOL` | `byte` | Push boolean (`0x01` = true, `0x00` = false) |
| `0x14` | `PUSH_DEC` | `f64 (LE)` | Push 64-bit float constant |
| `0x15` | `LOAD` | `len, name` | Load variable `name` and push its value |
| `0x16` | `LOAD_STR` | `len, name` | Load string variable (spica only; hydrae emits `0x15` for all loads) |
| `0x17` | `LOAD_BOOL` | `len, name` | Load boolean variable |
| `0x18` | `LOAD_DEC` | `len, name` | Load float variable |

#### Comparison Operations

| Opcode | Mnemonic | Operand | Description |
|--------|----------|---------|-------------|
| `0xC0` | `COMPARE` | `op_byte` | Pop two integers, compare with operator, push Bool result |
| `0xC1` | `JUMP_IF` | `len, label` | Pop Bool; jump to `label` if true, else skip |
| `0xC2` | `CALL_IF` | `len, label` | Pop Bool; call `label` if true, else skip |

Comparison operator bytes (second byte of COMPARE instruction):

| Byte | Operator |
|------|----------|
| `0xC3` | `==` (EQ) |
| `0xC4` | `>=` (GE) |
| `0xC5` | `>` (GT) |
| `0xC6` | `<=` (LE) |
| `0xC7` | `<` (LT) |
| `0xC8` | `!=` (NE) |

COMPARE only works on `Int` values. Comparing other types will panic with `KernelError: COMPARE expects integers`.

#### Logical Operations

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| `0xD0` | `AND` | Pop two Bools, push `a && b` |
| `0xD1` | `OR` | Pop two Bools, push `a \|\| b` |
| `0xD2` | `NOT` | Pop one Bool, push `!a` |
| `0xD3` | `XOR` | Pop two Bools, push `a ^ b` |

All logical operations require Bool values. Passing non-Bool values panics.

#### Control Flow

| Opcode | Mnemonic | Operand | Description |
|--------|----------|---------|-------------|
| `0xD4` | `LABEL` | `len, name` | Label marker; skipped during execution (pre-scanned) |
| `0xE2` | `CALL` | `len, label` | Push return address, jump to label |
| `0xE3` | `RET` | — | Pop return address, jump to it |
| `0xE4` | `JUMP` | `len, label` | Unconditional jump to label |

#### I/O

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| `0xE0` | `PRINT` | Pop top of stack, print it to stdout (with newline) |
| `0xE1` | `INPUT` | Read line from stdin, push as String |

#### Structural Markers

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| `0xF0` | `ARC_START` | Program start marker (must be byte 0) |
| `0xF1` | `ARC_END` | Program end marker; terminates the VM |
| `0xF2` | `ARC_DELIM` | Block delimiter; no-op at runtime |
| `0x00` | *(NOP)* | No operation; advances ip by 1 |

---

## AVM Assembly Reference

The AVM assembly format is a plain-text, line-oriented representation of the bytecode. Each line is one instruction; operands follow the mnemonic separated by whitespace.

```
ARC_START
PUSH_INT 10
PUSH_INT 32
ADD 2
PRINT
ARC_END
```

Mnemonics are case-sensitive and must be uppercase. Blank lines are skipped. There are no comments in the AVM assembly format.

String and label operands that contain spaces are supported — the assembler joins all tokens after the mnemonic with a space. This means label names can technically contain spaces, though this is not recommended.

**Full mnemonic list:**

```
ADD <n>          SUB <n>          MUL <n>         DIV <n>         MOD <n>
STORE <name>     PUSH_INT <val>   PUSH_STR <val>  PUSH_BOOL <val> PUSH_DEC <val>
LOAD <name>      LOAD_INT <name>  LOAD_STR <name> LOAD_BOOL <name> LOAD_DEC <name>
COMPARE <op>     JUMP_IF <label>  CALL_IF <label>
AND              OR               NOT             XOR
LABEL <name>     CALL <label>     RET             JUMP <label>
PRINT            INPUT
ARC_START        ARC_END          ARC_DELIM
```

`COMPARE` takes a string operator: `EQ`, `GT`, `GE`, `LT`, `LE`, `NE`.

`PUSH_BOOL` takes `true` or `false` (lowercase).

---

## Language Quirks and Gotchas

**No operator precedence in `let`.**
Arithmetic in `let` expressions is strictly left-to-right. `let x = 2 + 3 * 4;` evaluates as `(2 + 3) * 4 = 20`. Use intermediate variables if you need explicit ordering.

**`#` comments are block-scoped.**
The `#` character puts the lexer into `Inactive` state, where it ignores everything until the next `#`. This means `#` does not just comment out a single line — it comments out everything up to the next hash. Be careful when using `#` in multi-line blocks.

**All variables are global.**
There is no scoping, shadowing, or stack frames for local variables. Functions share the same flat variable store as the caller. This makes recursion dangerous — recursive calls will overwrite the caller's variables.

**`input` always yields a string.**
There is no way to read an integer or boolean directly from stdin. If your program needs numeric input, you will need to pre-format your input or work around this limitation at a higher level.

**Arithmetic operations are integer-only at the VM level.**
`ADD`, `SUB`, `MUL`, `DIV`, `MOD` all require `Int` values. You can store and load floats, but you cannot add two floats — the VM will panic with `KernelError`.

**Labels are byte-offset based internally.**
The label table maps label names to byte offsets (the position of the `LABEL` instruction itself in the bytecode). When a jump resolves to a label, it sets `ip` to the byte of the `LABEL` instruction, and the instruction immediately skips it and continues. This works correctly but means you cannot jump "into the middle" of an instruction.

**`%define` is for x86-64 mode and is token-level only.**
`%define` is intended for use with `-x` (x86-64 output) to assign names to memory addresses. Substitution only replaces `Token::Identifier` tokens — not keywords, punctuation, or literals. You cannot use `%define` to alias a keyword or rename a literal value.

**File extensions are enforced.**
hydrae will panic with `UsageError` if the input file does not end in `.arc`. spica has no such check.

---

## Error Reference

Errors in ARCTURUS follow a `Category: Message` format. Here is a summary of the error types you may encounter:

| Prefix | Source | Meaning |
|--------|--------|---------|
| `KernelError` | muphrid / hydrae | Internal VM/runtime error, usually a type mismatch or missing data |
| `SyntaxError` | hydrae parser | Invalid ARC syntax |
| `CompileError` | hydrae | File I/O or invalid compiler invocation |
| `UsageError` | hydrae | Wrong command-line usage |
| `AssemblySyntaxError` | hydrae / spica | Unknown mnemonic in assembly pass |
| `AssemblerError` | hydrae / spica | Invalid operand during assembly |
| `DisassemblerError` | spica | Unknown opcode during disassembly |
| `SerializationError` | hydrae | Invalid node during serialization to assembly text |

---

## End-to-End Examples

### Hello World

```arc
{
    print "Hello, World!";
}
```

Compile and run:

```sh
./hydrae hello.arc
./muphrid hello.avm
```

### Counting Loop

```arc
{
    let i = 0;
    label loopTop;
    jumpif loopEnd i >= 5;
    print i;
    let i = i + 1;
    jump loopTop;
    label loopEnd;
    print "done";
}
```

### Function Call

```arc
{
    let x = 10;
    call double;
    print result;
    exit;
}{
    label double;
    let result = x + x;
    return;
}
```

### Conditional with Multiple Blocks

```arc
{
    input userInput;
    print "You typed:";
    print userInput;
}
```

### Using %define and %scope

`%define` is most useful in x86-64 mode to give readable names to memory addresses. `%scope` works in both modes.

```arc
# main.arc (x86-64 mode)
%define BUFFER as x1000
%define COUNTER as x2000
%scope helpers.arc

{
    jump BUFFER;
}
```

```arc
# helpers.arc
{
    label printI;
    print i;
    return;
}
```

### Working at the Assembly Level with spica

Write raw AVM assembly:

```
ARC_START
PUSH_INT 6
PUSH_INT 7
MUL 2
PRINT
ARC_END
```

Assemble and run:

```sh
./spica assemble math.s math.avm
./muphrid math.avm
# prints: 42
```

Disassemble compiler output to inspect it:

```sh
./hydrae program.arc
./spica disassemble program.avm program_dis.s
cat program_dis.s
```

### Inspecting Compilation Stages

```sh
# Stop after lexing — see the token stream
./hydrae program.arc -l
cat program.avm

# Stop after parsing — see the AST
./hydrae program.arc -p
cat program.avm

# Stop after serialization — see the AVM assembly text
./hydrae program.arc -s
cat program.avm

# Full debug output for each stage
./hydrae program.arc -d

# Compile to x86-64 NASM assembly
./hydrae program.arc -x
cat program.asm
```
