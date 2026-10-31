package semantic

import (
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
)

type Program struct {
	Root      ast.Stmt
	ExprTypes map[ast.Span]Type
	Symbols   *SymbolTable
	Functions map[string]FunctionSig
}
type Analyzer struct {
	ExprTypes   map[ast.Span]Type
	Symbols     *SymbolTable
	Functions   map[string]FunctionSig
	Diagnostics []Diagnostic
	current     *FunctionSig
}

func NewAnalyzer() *Analyzer {
	a := &Analyzer{ExprTypes: map[ast.Span]Type{}, Symbols: NewSymbolTable(), Functions: map[string]FunctionSig{}}
	a.registerBuiltins()
	return a
}
func Analyze(root ast.Stmt) (*Program, []Diagnostic) {
	a := NewAnalyzer()
	a.collect(root)
	WalkStmt(a, root)
	return &Program{Root: root, ExprTypes: a.ExprTypes, Symbols: a.Symbols, Functions: a.Functions}, a.Diagnostics
}
func (a *Analyzer) registerBuiltins() {
	a.Functions["length"] = FunctionSig{Name: "length", Params: []Type{{Kind: Unknown}}, Result: Type{Kind: Int}, Builtin: true}
}
func (a *Analyzer) collect(s ast.Stmt) {
	switch x := s.(type) {
	case ast.Block:
		for _, s := range x.Statements {
			a.collect(s)
		}
	case ast.Function:
		if _, ok := a.Functions[x.Name]; ok {
			a.Diagnostics = append(a.Diagnostics, errorAt("E3011", fmt.Sprintf("duplicate function `%s`", x.Name), x.S))
		} else {
			params := make([]Type, len(x.Params))
			for n, p := range x.Params {
				params[n] = FromAST(p.Type)
			}
			result := FromAST(x.ReturnType)
			a.Functions[x.Name] = FunctionSig{Name: x.Name, Params: params, Result: result, Span: x.S}
		}
	}
}
func (a *Analyzer) record(e ast.Expr, t Type) Type { a.ExprTypes[e.Span()] = t; return t }
func (a *Analyzer) diag(code, msg string, span ast.Span) {
	a.Diagnostics = append(a.Diagnostics, errorAt(code, msg, span))
}
func (a *Analyzer) VisitLiteral(x ast.Literal) Type { return a.record(x, FromValue(x.V)) }
func (a *Analyzer) VisitVariable(x ast.Variable) Type {
	b, ok := a.Symbols.Lookup(x.Name)
	if !ok {
		a.diag("E3001", fmt.Sprintf("variable `%s` used before initialization", x.Name), x.S)
		return a.record(x, Type{Kind: Unknown})
	}
	return a.record(x, b.Type)
}
func (a *Analyzer) VisitArray(x ast.Array) Type {
	t := Type{Kind: Unknown}
	for _, e := range x.Elements {
		t = Merge(t, WalkExpr(a, e))
	}
	return a.record(x, Type{Kind: Array, Element: typePtr(t)})
}
func (a *Analyzer) VisitMap(x ast.Map) Type {
	t := Type{Kind: Unknown}
	for _, e := range x.Entries {
		t = Merge(t, WalkExpr(a, e.Value))
	}
	return a.record(x, Type{Kind: Map, Key: typePtr(Type{Kind: String}), Value: typePtr(t)})
}
func (a *Analyzer) VisitIndex(x ast.Index) Type {
	target := WalkExpr(a, x.Target)
	key := WalkExpr(a, x.Key)
	switch target.Kind {
	case Array:
		if key.Kind != Int && key.Kind != Unknown {
			a.diag("E3006", "array index must be Int", x.Key.Span())
		}
		return a.record(x, *target.Element)
	case Map:
		if !Compatible(*target.Key, key) {
			a.diag("E3006", fmt.Sprintf("map index must be %s, found %s", target.Key, key), x.Key.Span())
		}
		return a.record(x, *target.Value)
	case Unknown:
		return a.record(x, Type{Kind: Unknown})
	default:
		a.diag("E3006", fmt.Sprintf("cannot index value of type %s", target), x.S)
		return a.record(x, Type{Kind: Unknown})
	}
}
func (a *Analyzer) VisitCall(x ast.Call) Type {
	sig, ok := a.Functions[x.Name]
	if !ok {
		a.diag("E3007", fmt.Sprintf("unknown function `%s`", x.Name), x.S)
		for _, arg := range x.Args {
			WalkExpr(a, arg)
		}
		return a.record(x, Type{Kind: Unknown})
	}
	if sig.Builtin && len(x.Args) != 1 {
		a.diag("E3012", "length expects one argument", x.S)
	}
	if !sig.Builtin && len(sig.Params) != len(x.Args) {
		a.diag("E3008", fmt.Sprintf("function `%s` expects %d arguments, got %d", x.Name, len(sig.Params), len(x.Args)), x.S)
	}
	for n, arg := range x.Args {
		actual := WalkExpr(a, arg)
		if n < len(sig.Params) && sig.Builtin {
			if actual.Kind != String && actual.Kind != Array && actual.Kind != Map && actual.Kind != Unknown {
				a.diag("E3012", "length requires a String, Array, or Map", arg.Span())
			}
		} else if n < len(sig.Params) && !Compatible(sig.Params[n], actual) {
			a.diag("E3003", fmt.Sprintf("type mismatch: expected %s, found %s", sig.Params[n], actual), arg.Span())
		}
	}
	return a.record(x, sig.Result)
}
func (a *Analyzer) VisitUnary(x ast.Unary) Type {
	t := WalkExpr(a, x.Value)
	if x.Op == ast.Neg && (t.IsNumeric() || t.Kind == Unknown) {
		return a.record(x, t)
	}
	a.diag("E3004", "unary minus requires a numeric operand", x.S)
	return a.record(x, Type{Kind: Unknown})
}
func (a *Analyzer) VisitBinary(x ast.Binary) Type {
	left, right := WalkExpr(a, x.Left), WalkExpr(a, x.Right)
	if left.Kind == Unknown || right.Kind == Unknown {
		return a.record(x, Type{Kind: Unknown})
	}
	switch x.Op {
	case ast.Add:
		if left.Kind == String && right.Kind == String {
			return a.record(x, left)
		}
		fallthrough
	case ast.Sub, ast.Mul, ast.Div:
		if left.IsNumeric() && right.IsNumeric() {
			return a.record(x, Merge(left, right))
		}
	case ast.Equal:
		if Compatible(left, right) {
			return a.record(x, Type{Kind: Bool})
		}
	case ast.And, ast.Or:
		if left.Kind == Bool && right.Kind == Bool {
			return a.record(x, Type{Kind: Bool})
		}
	}
	a.diag("E3004", fmt.Sprintf("invalid operation %s for %s and %s", x.Op, left, right), x.S)
	return a.record(x, Type{Kind: Unknown})
}
func (a *Analyzer) VisitBlock(x ast.Block) Flow {
	flow := Flow{CanFallThrough: true}
	for _, s := range x.Statements {
		next := WalkStmt(a, s)
		if flow.CanFallThrough {
			flow = Flow{CanFallThrough: next.CanFallThrough, CanReturn: flow.CanReturn || next.CanReturn}
		} else {
			flow.CanReturn = flow.CanReturn || next.CanReturn
		}
	}
	return flow
}
func (a *Analyzer) VisitLet(x ast.Let) Flow {
	actual := WalkExpr(a, x.Value)
	declared := actual
	if x.Annotation != nil {
		declared = FromAST(x.Annotation)
		if !Compatible(declared, actual) {
			a.diag("E3003", fmt.Sprintf("type mismatch: expected %s, found %s", declared, actual), x.S)
		}
	}
	if !a.Symbols.Declare(Binding{Name: x.Name, Type: declared, Mutable: true, Span: x.S}) {
		a.diag("E3002", fmt.Sprintf("variable `%s` is already declared in this scope", x.Name), x.S)
	}
	return Flow{CanFallThrough: true}
}
func (a *Analyzer) VisitAssign(x ast.Assign) Flow {
	b, ok := a.Symbols.Lookup(x.Name)
	actual := WalkExpr(a, x.Value)
	if !ok {
		a.diag("E3001", fmt.Sprintf("variable `%s` used before initialization", x.Name), x.S)
	} else if !Compatible(b.Type, actual) {
		a.diag("E3003", fmt.Sprintf("type mismatch: expected %s, found %s", b.Type, actual), x.S)
	}
	return Flow{CanFallThrough: true}
}
func (a *Analyzer) VisitPrint(x ast.Print) Flow {
	WalkExpr(a, x.Value)
	return Flow{CanFallThrough: true}
}
func (a *Analyzer) VisitAssert(x ast.Assert) Flow {
	t := WalkExpr(a, x.Value)
	if t.Kind != Bool && t.Kind != Unknown {
		a.diag("E3005", "assert condition must be Bool", x.Value.Span())
	}
	return Flow{CanFallThrough: true}
}
func (a *Analyzer) VisitReturn(x ast.Return) Flow {
	if a.current == nil {
		a.diag("E3009", "return is only valid inside a function", x.S)
		return Flow{CanFallThrough: false, CanReturn: true}
	}
	actual := Type{Kind: Null}
	if x.HasValue {
		actual = WalkExpr(a, x.Value)
	}
	if !Compatible(a.current.Result, actual) {
		a.diag("E3009", fmt.Sprintf("type mismatch: function `%s` returns %s, found %s", a.current.Name, a.current.Result, actual), x.S)
	}
	return Flow{CanFallThrough: false, CanReturn: true}
}
func (a *Analyzer) VisitFunction(x ast.Function) Flow {
	sig := a.Functions[x.Name]
	saved := a.current
	a.current = &sig
	a.Symbols.Push()
	defer func() { a.current = saved; a.Symbols.Pop() }()
	for n, p := range x.Params {
		if !a.Symbols.Declare(Binding{Name: p.Name, Type: FromAST(p.Type), Mutable: true, Span: x.S}) {
			a.diag("E3002", fmt.Sprintf("duplicate parameter `%s`", p.Name), x.S)
		}
		_ = n
	}
	flow := a.VisitBlock(ast.Block{Statements: x.Body, S: x.S})
	if sig.Result.Kind != Unknown && sig.Result.Kind != Null && flow.CanFallThrough {
		a.diag("E3010", fmt.Sprintf("function `%s` can fall through without returning", x.Name), x.S)
	}
	return Flow{CanFallThrough: true}
}
func (a *Analyzer) VisitIf(x ast.If) Flow {
	condition := WalkExpr(a, x.Condition)
	if condition.Kind != Bool && condition.Kind != Unknown {
		a.diag("E3005", "if condition must be Bool", x.Condition.Span())
	}
	a.Symbols.Push()
	yes := a.VisitBlock(ast.Block{Statements: x.Then, S: x.S})
	a.Symbols.Pop()
	if x.Otherwise == nil {
		return Flow{CanFallThrough: true, CanReturn: yes.CanReturn}
	}
	a.Symbols.Push()
	no := a.VisitBlock(ast.Block{Statements: x.Otherwise, S: x.S})
	a.Symbols.Pop()
	return Flow{CanFallThrough: yes.CanFallThrough || no.CanFallThrough, CanReturn: yes.CanReturn || no.CanReturn}
}
func (a *Analyzer) VisitRepeat(x ast.Repeat) Flow {
	count := WalkExpr(a, x.Count)
	if !count.IsNumeric() && count.Kind != Unknown {
		a.diag("E3005", "repeat count must be numeric", x.Count.Span())
	}
	a.Symbols.Push()
	a.VisitBlock(ast.Block{Statements: x.Body, S: x.S})
	a.Symbols.Pop()
	return Flow{CanFallThrough: true}
}
func (a *Analyzer) VisitWindow(x ast.Window) Flow {
	for _, e := range []ast.Expr{x.Width, x.Height} {
		t := WalkExpr(a, e)
		if !t.IsNumeric() && t.Kind != Unknown {
			a.diag("E3005", "window dimensions must be numeric", e.Span())
		}
	}
	return Flow{CanFallThrough: true}
}
