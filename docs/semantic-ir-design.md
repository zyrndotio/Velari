# Go semantic analysis and typed IR design

## Purpose

This phase turns a parsed `ast.Stmt` tree into a checked, typed program and then into a backend-neutral intermediate representation. The semantic pass is the gate between syntax and execution: the interpreter, WebAssembly adapter, native backend, and desktop host should all consume the same validated model.

The first implementation preserves the established semantic and IR behavior, while correcting two limitations in the prototype Go AST: collection types must have an explicit canonical kind, and IR control flow must represent loop back-edges rather than treating a repeat count as a boolean condition.

## Package layout

```text
go/semantic/
    types.go       canonical type algebra and compatibility rules
    symbols.go     lexical scopes and function signatures
    analyzer.go    two-pass declaration and expression checking
    diagnostics.go structured semantic diagnostics

go/ir/
    types.go       IR type and value identifiers
    instruction.go typed instructions and terminators
    module.go      functions, basic blocks, and module validation
    lower.go       AST-to-IR lowering
    printer.go     deterministic textual IR output

go/compiler/
    pipeline.go    lex -> parse -> analyze -> lower orchestration
```

`go/semantic` and `go/ir` must import `go/ast`, but must not import the interpreter, CLI, filesystem, operating-system packages, Wails, Tauri, or `syscall/js`.

## Semantic model

Use one canonical semantic type algebra. It should not duplicate equivalent type systems in the AST and analyzer.

```go
type TypeKind uint8
const (
    Invalid TypeKind = iota
    Unknown
    Int
    Float
    Bool
    String
    Null
    Array
    Map
)

type Type struct {
    Kind TypeKind
    Element *Type       // Array
    Key, Value *Type    // Map
}

type FunctionSig struct {
    Name string
    Params []Type
    Result Type
    Span ast.Span
}
```

`ast.Type` can remain the parser-facing syntax representation in this phase, but `semantic.Type` should be the canonical checked type. Add explicit `KindArray` and `KindMap` constants to the AST value/type model before lowering; do not rely on empty `Kind` plus a non-nil pointer or string values such as `"Array"`.

The type lattice is:

- `Unknown` is an error-recovery type. It suppresses cascaded diagnostics but is never a successful concrete result.
- `Int` widens to `Float` where a numeric value is expected.
- `Null` is only compatible with `Null` in the initial language.
- `Array<T>` is compatible only with `Array<U>` when `T` and `U` are compatible.
- `Map<K,V>` is compatible only when both key and value types are compatible.
- An unannotated empty array or map has an unknown element/value type and should require an annotation when used in a context that needs a concrete type.

The analyzer should expose:

```go
func Analyze(root ast.Stmt) (*Program, []Diagnostic)

type Program struct {
    Root ast.Stmt
    ExprTypes map[ast.Expr]Type // or node IDs after AST IDs are added
    Symbols *SymbolTable
    Functions map[string]FunctionSig
}
```

Using `map[ast.Expr]Type` is acceptable for a short-lived prototype only if all AST nodes remain comparable. The preferred design adds a stable `NodeID uint32` to expression and statement nodes and stores types by ID, avoiding interface-comparability and pointer-identity problems.

## Analysis passes

### Pass 0: source and AST validation

The compiler pipeline should reject lexer and parser diagnostics before semantic analysis. The analyzer assumes a structurally valid tree but still uses `Unknown` recovery for malformed or partially constructed nodes.

### Pass 1: collect declarations

Walk the program and collect all top-level functions before checking bodies. A function signature contains its parameter types and result type. Unannotated parameters and results are `Unknown` for now. Duplicate function names are errors. The analyzer should also reject duplicate parameters in one function.

Built-ins are registered in the same function environment. The initial built-in set is:

```text
length(String) -> Int
length(Array<T>) -> Int
length(Map<K,V>) -> Int
```

A later standard-library phase can replace the hard-coded built-in with an imported signature table.

### Pass 2: check statements and expressions

Maintain a stack of lexical scopes. A scope maps names to a binding containing the declared type, declaration span, and mutability metadata. The first Go implementation can keep the current assignment semantics, where `let` bindings are assignable, but the binding structure should reserve a `Mutable` flag.

Required checks:

- variable use and assignment must resolve to an initialized binding;
- redeclaration in the same scope is an error;
- assignment must be compatible with the binding type;
- `let x: T be value` checks `value` against `T` before defining `x`;
- `if` and `assert` conditions must be `Bool`;
- `repeat` counts and window dimensions must be `Int` or `Float`;
- `return` is legal only in a function and must match its declared result;
- a function with a declared non-`Null` result must return on every reachable path;
- calls must resolve, have the correct arity, and have compatible arguments;
- array indexes must be `Int`;
- map indexes must match the map key type;
- arithmetic requires numeric operands, except `String + String`;
- equality returns `Bool` and requires compatible operands;
- `and` and `or` require two `Bool` operands.

Expression checking returns a type even after an error where possible. For example, an undefined variable produces one diagnostic and returns `Unknown`, allowing the surrounding binary expression to continue without producing a second misleading error.

### Control-flow return analysis

`statementReturns` must mean “all paths through this statement return,” not “some statement nested inside returns.” Use a small control-flow result:

```go
type Flow struct {
    CanFallThrough bool
    CanReturn bool
}
```

Examples:

- `return` => `{CanFallThrough:false, CanReturn:true}`
- `print x` => `{true,false}`
- a block combines statements in order and stops composing after a statement that cannot fall through;
- `if` without `otherwise` can fall through;
- `if` with both branches returns only if both branches cannot fall through;
- `repeat` can always fall through because the count may be zero.

This avoids accepting a function merely because one branch contains a return.

## Diagnostics

Use a public diagnostic type shared by semantic analysis and later IR validation:

```go
type Diagnostic struct {
    Code string
    Severity Severity
    Message string
    Span ast.Span
    Notes []Note
}
```

Suggested codes:

```text
E3001 undefined variable
E3002 duplicate declaration
E3003 type mismatch
E3004 invalid operator operands
E3005 invalid condition type
E3006 invalid index type
E3007 unknown function
E3008 wrong argument count
E3009 invalid return
E3010 missing return
E3011 duplicate function
E3012 invalid builtin call
E3901 internal semantic invariant
E4001 invalid IR
```

Diagnostics should point to the smallest useful source span: the variable use for undefined names, the annotation for declaration mismatches, the operator span for invalid operations, and the call span for arity errors. Preserve source filename and line text at the compiler boundary rather than embedding filesystem access in the semantic package.

## Typed IR design

The IR should be a typed, control-flow-aware, backend-neutral module. It is not yet machine code and should not expose Go runtime values as its primary representation.

```go
type ValueID uint32
type BlockID uint32

type Module struct { Functions []Function }
type Function struct {
    Name string
    Params []Param
    Result semantic.Type
    Entry BlockID
    Blocks []BasicBlock
}
type Param struct { Name string; Value ValueID; Type semantic.Type }
type BasicBlock struct {
    ID BlockID
    Instructions []Instruction
    Terminator Terminator
}
```

Each instruction that produces a value has exactly one `Result ValueID` and a declared `Type`. A value ID is defined exactly once and may be used only after its definition in the same function. Block parameters are optional in phase one; use explicit `Phi` instructions only when branch-merged values become necessary.

### Instructions

The initial instruction set should be:

```go
type Instruction interface { instruction(); Result() (ValueID, bool); Type() semantic.Type }

Const       { ResultID ValueID; Value ast.Value; ValueType semantic.Type }
Load        { ResultID ValueID; Name string; ValueType semantic.Type }
Store       { Name string; Value ValueID; ValueType semantic.Type }
Unary       { ResultID ValueID; Op ast.Op; Operand ValueID; ValueType semantic.Type }
Binary      { ResultID ValueID; Op ast.Op; Left, Right ValueID; ValueType semantic.Type }
Call        { ResultID ValueID; Name string; Args []ValueID; ValueType semantic.Type }
Index       { ResultID ValueID; Target, Index ValueID; ValueType semantic.Type }
MakeArray   { ResultID ValueID; Elements []ValueID; ValueType semantic.Type }
MakeMap     { ResultID ValueID; Keys []string; Values []ValueID; ValueType semantic.Type }
Print       { Value ValueID }
Assert      { Value ValueID }
Window      { Title string; Width, Height ValueID }
```

`Store` is deliberately name-based in phase one so the tree-walking interpreter and a simple backend can share semantics. A later SSA conversion can replace names with address slots or explicit local declarations.

### Terminators

Every basic block ends in exactly one terminator:

```go
Return { Value *ValueID }
Jump   { Target BlockID }
Branch { Condition ValueID; Then, Else BlockID }
Unreachable {}
```

There is no fall-through by omission. The IR validator must reject blocks without terminators and branches whose condition is not `Bool`.

### Lowering rules

Lower expressions recursively and return `(ValueID, semantic.Type)`. The lowering pass consumes the typed `Program`, not the raw AST, so it can trust operator and index types after semantic validation.

- literals become `Const`;
- variables become `Load`;
- declarations and assignments become `Store`;
- arithmetic and logical expressions become typed `Unary`/`Binary`;
- calls become `Call` with a known result type;
- arrays/maps become `MakeArray`/`MakeMap`;
- indexing becomes `Index`;
- `print`, `assert`, and `window` become effect instructions;
- functions are lowered independently, with a synthetic `main` for top-level non-function statements.

For `if`, create `then`, `else`, and `join` blocks and terminate the current block with `Branch`. Each branch jumps to `join` unless it already returns. For `repeat count times`, create `header`, `body`, and `exit` blocks with an explicit loop counter:

```text
counter = 0
jump header
header:
    cond = counter < count
    branch cond, body, exit
body:
    ...
    counter_next = counter + 1
    counter = counter_next
    jump header
exit:
```

The earlier lowerer emitted a branch directly on the iteration count and does not model the decrement/back-edge; the Go IR should not carry that bug forward.

## IR validation

Run validation immediately after lowering and before any backend consumes the module. Validate:

- every referenced block exists;
- every block has one terminator;
- every value ID has one definition;
- every use refers to a defined value in the function;
- instruction operand types match the instruction contract;
- `Store` assignments match the binding type;
- `Return` matches the function result type;
- `Branch` uses `Bool`;
- `Index` uses `Int` for arrays and the declared map key type for maps;
- all function calls resolve and match signature arity/types;
- no instruction contains an `Unknown` type after successful semantic analysis.

Validation errors are compiler bugs or incomplete lowering, not user diagnostics; use `E4001` and include the function/block/value context.

## Compiler pipeline API

The public orchestration layer should be small:

```go
type Result struct {
    AST ast.Stmt
    Typed *semantic.Program
    IR *ir.Module
    Diagnostics []Diagnostic
}

func Compile(source string, options Options) Result
```

`Compile` should stop after each failing stage. `check` requests AST plus semantic diagnostics; `run` can use the typed program with the interpreter during the transition; `ir` requests the validated module and deterministic text output; future `build --target wasm` selects a backend after IR validation.

The runtime should initially remain an AST/typed-program interpreter. Once the IR has a stable validator, add an IR interpreter as a conformance backend and compare its output against the existing runtime on the example and regression corpus.

## Implementation sequence

1. Normalize collection kinds and add node IDs/spans where needed.
2. Create `go/semantic/types.go`, compatibility, type formatting, and built-in signatures.
3. Port symbol scopes and function collection from the established analyzer.
4. Port expression and statement checks with the improved all-path return analysis.
5. Add semantic regression tests for every established semantic rule and for unknown-name recovery.
6. Create `go/ir` value IDs, instruction types, blocks, terminators, and validator.
7. Lower literals, variables, stores, arithmetic, calls, collections, print, assert, and return.
8. Lower conditionals and counted loops into explicit CFGs.
9. Add deterministic IR text output and CLI `ir` mode.
10. Add an IR interpreter and differential tests against the AST interpreter.
11. Only then start a native or WASM code-generation backend.

## Acceptance criteria

This phase is complete when:

- all existing valid examples pass semantic analysis;
- invalid annotations, calls, indexing, conditions, returns, and assignments produce stable diagnostics;
- `go test ./...` includes semantic and IR validator coverage;
- `velari check` runs the semantic pass rather than only lexing/parsing;
- `velari ir examples/typed.vr` prints deterministic typed IR;
- malformed or incomplete ASTs do not panic the analyzer;
- the same validated module can be handed to an AST interpreter, IR interpreter, and future WASM/native backend;
- `GOOS=js GOARCH=wasm go test ./...` remains free of OS-specific core dependencies.
