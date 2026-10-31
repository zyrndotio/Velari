package ast

import "fmt"

type Span struct{ Start, End, Line, Column int }
type TypeKind string

const (
	TypeUnknown TypeKind = "Unknown"
	TypeInt     TypeKind = "Int"
	TypeFloat   TypeKind = "Float"
	TypeBool    TypeKind = "Bool"
	TypeString  TypeKind = "String"
	TypeNull    TypeKind = "Null"
	TypeArray   TypeKind = "Array"
	TypeMap     TypeKind = "Map"
)

type Type struct {
	Kind       TypeKind
	Element    *Type
	Key, Value *Type
}

func (t Type) String() string {
	if t.Element != nil {
		return fmt.Sprintf("Array<%s>", t.Element)
	}
	if t.Key != nil {
		return fmt.Sprintf("Map<%s, %s>", t.Key, t.Value)
	}
	if t.Kind == "" {
		return string(TypeUnknown)
	}
	return string(t.Kind)
}

type Value struct {
	Kind   TypeKind
	Number float64
	Bool   bool
	String string
	Array  []Value
	Map    map[string]Value
}

func (v Value) Display() string {
	switch v.Kind {
	case TypeFloat, TypeInt:
		return fmt.Sprintf("%g", v.Number)
	case TypeBool:
		if v.Bool {
			return "true"
		}
		return "false"
	case TypeString:
		return v.String
	case TypeNull:
		return "null"
	case TypeArray:
		s := "["
		for i, x := range v.Array {
			if i > 0 {
				s += ", "
			}
			s += x.Display()
		}
		return s + "]"
	case TypeMap:
		s := "{"
		first := true
		for k, x := range v.Map {
			if !first {
				s += ", "
			}
			first = false
			s += k + ": " + x.Display()
		}
		return s + "}"
	}
	return "null"
}

type Op string

const (
	Add   Op = "+"
	Sub   Op = "-"
	Mul   Op = "*"
	Div   Op = "/"
	Equal Op = "=="
	And   Op = "and"
	Or    Op = "or"
	Neg   Op = "neg"
)

type Expr interface {
	exprNode()
	Span() Span
}
type Literal struct {
	V Value
	S Span
}

func (Literal) exprNode()    {}
func (e Literal) Span() Span { return e.S }

type Variable struct {
	Name string
	S    Span
}

func (Variable) exprNode()    {}
func (e Variable) Span() Span { return e.S }

type Array struct {
	Elements []Expr
	S        Span
}

func (Array) exprNode()    {}
func (e Array) Span() Span { return e.S }

type Map struct {
	Entries []MapEntry
	S       Span
}
type MapEntry struct {
	Key   string
	Value Expr
}

func (Map) exprNode()    {}
func (e Map) Span() Span { return e.S }

type Index struct {
	Target, Key Expr
	S           Span
}

func (Index) exprNode()    {}
func (e Index) Span() Span { return e.S }

type Binary struct {
	Left  Expr
	Op    Op
	Right Expr
	S     Span
}

func (Binary) exprNode()    {}
func (e Binary) Span() Span { return e.S }

type Unary struct {
	Op    Op
	Value Expr
	S     Span
}

func (Unary) exprNode()    {}
func (e Unary) Span() Span { return e.S }

type Call struct {
	Name string
	Args []Expr
	S    Span
}

func (Call) exprNode()    {}
func (e Call) Span() Span { return e.S }

type Stmt interface {
	stmtNode()
	Span() Span
}
type Block struct {
	Statements []Stmt
	S          Span
}

func (Block) stmtNode()    {}
func (s Block) Span() Span { return s.S }

type Let struct {
	Name       string
	Annotation *Type
	Value      Expr
	S          Span
}

func (Let) stmtNode()    {}
func (s Let) Span() Span { return s.S }

type Assign struct {
	Name  string
	Value Expr
	S     Span
}

func (Assign) stmtNode()    {}
func (s Assign) Span() Span { return s.S }

type Print struct {
	Value Expr
	S     Span
}

func (Print) stmtNode()    {}
func (s Print) Span() Span { return s.S }

type Assert struct {
	Value Expr
	S     Span
}

func (Assert) stmtNode()    {}
func (s Assert) Span() Span { return s.S }

type Return struct {
	Value    Expr
	HasValue bool
	S        Span
}

func (Return) stmtNode()    {}
func (s Return) Span() Span { return s.S }

type Function struct {
	Name       string
	Params     []Param
	ReturnType *Type
	Body       []Stmt
	S          Span
}
type Param struct {
	Name string
	Type *Type
}

func (Function) stmtNode()    {}
func (s Function) Span() Span { return s.S }

type If struct {
	Condition       Expr
	Then, Otherwise []Stmt
	S               Span
}

func (If) stmtNode()    {}
func (s If) Span() Span { return s.S }

type Repeat struct {
	Count Expr
	Body  []Stmt
	S     Span
}

func (Repeat) stmtNode()    {}
func (s Repeat) Span() Span { return s.S }

type Window struct {
	Title         string
	Width, Height Expr
	S             Span
}

func (Window) stmtNode()    {}
func (s Window) Span() Span { return s.S }
