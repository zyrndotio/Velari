package runtime

import (
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
	"io"
	"math"
)

type Error struct{ Message string }

func (e Error) Error() string { return e.Message }

type function struct {
	params []ast.Param
	body   []ast.Stmt
}
type flow struct {
	returned bool
	value    ast.Value
}
type Interpreter struct {
	out       io.Writer
	scopes    []map[string]ast.Value
	functions map[string]function
}

func New(out io.Writer) *Interpreter {
	return &Interpreter{out: out, scopes: []map[string]ast.Value{{}}, functions: map[string]function{}}
}
func (i *Interpreter) Execute(root ast.Stmt) error {
	if b, ok := root.(ast.Block); ok {
		for _, s := range b.Statements {
			if f, ok := s.(ast.Function); ok {
				i.functions[f.Name] = function{f.Params, f.Body}
			}
		}
	}
	f, e := i.exec(root)
	if e != nil {
		return e
	}
	if f.returned {
		return Error{"return outside function"}
	}
	return nil
}
func (i *Interpreter) push() { i.scopes = append(i.scopes, map[string]ast.Value{}) }
func (i *Interpreter) pop()  { i.scopes = i.scopes[:len(i.scopes)-1] }
func (i *Interpreter) get(n string) (ast.Value, bool) {
	for x := len(i.scopes) - 1; x >= 0; x-- {
		if v, ok := i.scopes[x][n]; ok {
			return v, true
		}
	}
	return ast.Value{}, false
}
func (i *Interpreter) set(n string, v ast.Value) { i.scopes[len(i.scopes)-1][n] = v }
func (i *Interpreter) assign(n string, v ast.Value) bool {
	for x := len(i.scopes) - 1; x >= 0; x-- {
		if _, ok := i.scopes[x][n]; ok {
			i.scopes[x][n] = v
			return true
		}
	}
	return false
}
func (i *Interpreter) exec(s ast.Stmt) (flow, error) {
	switch x := s.(type) {
	case ast.Block:
		for _, s := range x.Statements {
			f, e := i.exec(s)
			if e != nil || f.returned {
				return f, e
			}
		}
	case ast.Function:
		return flow{}, nil
	case ast.Let:
		v, e := i.eval(x.Value)
		if e != nil {
			return flow{}, e
		}
		i.set(x.Name, v)
	case ast.Assign:
		v, e := i.eval(x.Value)
		if e != nil {
			return flow{}, e
		}
		if !i.assign(x.Name, v) {
			return flow{}, Error{fmt.Sprintf("undefined variable `%s`", x.Name)}
		}
	case ast.Print:
		v, e := i.eval(x.Value)
		if e != nil {
			return flow{}, e
		}
		fmt.Fprintln(i.out, v.Display())
	case ast.Assert:
		v, e := i.eval(x.Value)
		if e != nil {
			return flow{}, e
		}
		if v.Kind != ast.TypeBool || !v.Bool {
			return flow{}, Error{"assertion failed"}
		}
	case ast.Return:
		if !x.HasValue {
			return flow{true, ast.Value{Kind: ast.TypeNull}}, nil
		}
		v, e := i.eval(x.Value)
		return flow{true, v}, e
	case ast.If:
		v, e := i.eval(x.Condition)
		if e != nil {
			return flow{}, e
		}
		branch := x.Otherwise
		if v.Kind == ast.TypeBool && v.Bool {
			branch = x.Then
		}
		i.push()
		f, e := i.exec(ast.Block{Statements: branch})
		i.pop()
		return f, e
	case ast.Repeat:
		v, e := i.eval(x.Count)
		if e != nil || v.Kind != ast.TypeInt && v.Kind != ast.TypeFloat {
			return flow{}, Error{"repeat count must be a non-negative Number"}
		}
		if v.Number < 0 {
			return flow{}, Error{"repeat count must be a non-negative Number"}
		}
		for n := 0; n < int(v.Number); n++ {
			i.push()
			f, e := i.exec(ast.Block{Statements: x.Body})
			i.pop()
			if e != nil || f.returned {
				return f, e
			}
		}
	case ast.Window:
		w, e := i.eval(x.Width)
		if e != nil {
			return flow{}, e
		}
		h, e := i.eval(x.Height)
		if e != nil {
			return flow{}, e
		}
		fmt.Fprintf(i.out, "[Velari] window %q %sx%s\n", x.Title, w.Display(), h.Display())
	}
	return flow{}, nil
}
func (i *Interpreter) eval(e ast.Expr) (ast.Value, error) {
	switch x := e.(type) {
	case ast.Literal:
		return x.V, nil
	case ast.Variable:
		if v, ok := i.get(x.Name); ok {
			return v, nil
		}
		return ast.Value{}, Error{fmt.Sprintf("undefined variable `%s`", x.Name)}
	case ast.Array:
		xs := make([]ast.Value, len(x.Elements))
		for n, e := range x.Elements {
			v, er := i.eval(e)
			if er != nil {
				return ast.Value{}, er
			}
			xs[n] = v
		}
		return ast.Value{Kind: ast.TypeArray, Array: xs}, nil
	case ast.Map:
		m := map[string]ast.Value{}
		for _, e := range x.Entries {
			v, er := i.eval(e.Value)
			if er != nil {
				return ast.Value{}, er
			}
			m[e.Key] = v
		}
		return ast.Value{Kind: ast.TypeMap, Map: m}, nil
	case ast.Index:
		t, er := i.eval(x.Target)
		if er != nil {
			return ast.Value{}, er
		}
		k, er := i.eval(x.Key)
		if er != nil {
			return ast.Value{}, er
		}
		if t.Kind == ast.TypeArray && k.Kind != ast.TypeString {
			n := int(k.Number)
			if n >= 0 && n < len(t.Array) {
				return t.Array[n], nil
			}
			return ast.Value{}, Error{"array index out of bounds"}
		}
		if t.Kind == ast.TypeMap && k.Kind == ast.TypeString {
			if v, ok := t.Map[k.String]; ok {
				return v, nil
			}
			return ast.Value{}, Error{fmt.Sprintf("missing map key `%s`", k.String)}
		}
		return ast.Value{}, Error{"invalid index operation"}
	case ast.Call:
		if x.Name == "length" {
			if len(x.Args) != 1 {
				return ast.Value{}, Error{"length expects one argument"}
			}
			v, e := i.eval(x.Args[0])
			if e != nil {
				return ast.Value{}, e
			}
			n := 0
			if v.Kind == ast.TypeString {
				n = len([]rune(v.String))
			} else if v.Kind == ast.TypeArray {
				n = len(v.Array)
			} else if v.Kind == ast.TypeMap {
				n = len(v.Map)
			} else {
				return ast.Value{}, Error{"length requires a string, array, or map"}
			}
			return ast.Value{Kind: ast.TypeInt, Number: float64(n)}, nil
		}
		f, ok := i.functions[x.Name]
		if !ok {
			return ast.Value{}, Error{fmt.Sprintf("unknown function `%s`", x.Name)}
		}
		if len(f.params) != len(x.Args) {
			return ast.Value{}, Error{"function argument count mismatch"}
		}
		args := make([]ast.Value, len(x.Args))
		for n, a := range x.Args {
			v, e := i.eval(a)
			if e != nil {
				return ast.Value{}, e
			}
			args[n] = v
		}
		i.push()
		for n, p := range f.params {
			i.set(p.Name, args[n])
		}
		r, e := i.exec(ast.Block{Statements: f.body})
		i.pop()
		if e != nil {
			return ast.Value{}, e
		}
		if r.returned {
			return r.value, nil
		}
		return ast.Value{Kind: ast.TypeNull}, nil
	case ast.Unary:
		v, e := i.eval(x.Value)
		if e != nil {
			return ast.Value{}, e
		}
		if x.Op == ast.Neg && v.Kind != ast.TypeString && v.Kind != ast.TypeBool {
			return ast.Value{Kind: v.Kind, Number: -v.Number}, nil
		}
		return ast.Value{}, Error{"invalid unary operation"}
	case ast.Binary:
		a, e := i.eval(x.Left)
		if e != nil {
			return ast.Value{}, e
		}
		b, e := i.eval(x.Right)
		if e != nil {
			return ast.Value{}, e
		}
		return binary(x.Op, a, b)
	}
	return ast.Value{}, Error{"unknown expression"}
}
func binary(op ast.Op, a, b ast.Value) (ast.Value, error) {
	if op == ast.Equal {
		return ast.Value{Kind: ast.TypeBool, Bool: a.Display() == b.Display()}, nil
	}
	if op == ast.And || op == ast.Or {
		if a.Kind != ast.TypeBool || b.Kind != ast.TypeBool {
			return ast.Value{}, Error{"logical operators require booleans"}
		}
		v := a.Bool && b.Bool
		if op == ast.Or {
			v = a.Bool || b.Bool
		}
		return ast.Value{Kind: ast.TypeBool, Bool: v}, nil
	}
	if op == ast.Add && a.Kind == ast.TypeString && b.Kind == ast.TypeString {
		return ast.Value{Kind: ast.TypeString, String: a.String + b.String}, nil
	}
	if a.Kind == ast.TypeString || b.Kind == ast.TypeString {
		return ast.Value{}, Error{"arithmetic requires numbers"}
	}
	n := 0.0
	switch op {
	case ast.Add:
		n = a.Number + b.Number
	case ast.Sub:
		n = a.Number - b.Number
	case ast.Mul:
		n = a.Number * b.Number
	case ast.Div:
		if b.Number == 0 {
			return ast.Value{}, Error{"division by zero"}
		}
		n = a.Number / b.Number
	default:
		return ast.Value{}, Error{"invalid binary operation"}
	}
	k := ast.TypeFloat
	if math.Trunc(n) == n && a.Kind == ast.TypeInt && b.Kind == ast.TypeInt {
		k = ast.TypeInt
	}
	return ast.Value{Kind: k, Number: n}, nil
}
