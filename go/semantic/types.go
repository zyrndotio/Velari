package semantic

import (
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
)

type Kind uint8

const (
	Invalid Kind = iota
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
	Kind       Kind
	Element    *Type
	Key, Value *Type
}

func (t Type) String() string {
	switch t.Kind {
	case Array:
		return fmt.Sprintf("Array<%s>", t.Element)
	case Map:
		return fmt.Sprintf("Map<%s, %s>", t.Key, t.Value)
	case Invalid:
		return "Invalid"
	case Unknown:
		return "Unknown"
	default:
		return [...]string{"", "", "Int", "Float", "Bool", "String", "Null"}[t.Kind]
	}
}
func (t Type) IsNumeric() bool { return t.Kind == Int || t.Kind == Float }
func (t Type) Equal(o Type) bool {
	if t.Kind != o.Kind {
		return false
	}
	switch t.Kind {
	case Array:
		return t.Element != nil && o.Element != nil && t.Element.Equal(*o.Element)
	case Map:
		return t.Key != nil && o.Key != nil && t.Value != nil && o.Value != nil && t.Key.Equal(*o.Key) && t.Value.Equal(*o.Value)
	default:
		return true
	}
}
func Compatible(expected, actual Type) bool {
	if expected.Kind == Unknown || actual.Kind == Unknown {
		return true
	}
	if expected.Equal(actual) {
		return true
	}
	return expected.Kind == Float && actual.Kind == Int
}
func Merge(a, b Type) Type {
	if a.Kind == Unknown {
		return b
	}
	if b.Kind == Unknown || a.Equal(b) {
		return a
	}
	if a.IsNumeric() && b.IsNumeric() {
		if a.Kind == Float || b.Kind == Float {
			return Type{Kind: Float}
		}
		return Type{Kind: Int}
	}
	if a.Kind == Array && b.Kind == Array {
		return Type{Kind: Array, Element: typePtr(Merge(*a.Element, *b.Element))}
	}
	if a.Kind == Map && b.Kind == Map {
		return Type{Kind: Map, Key: typePtr(Merge(*a.Key, *b.Key)), Value: typePtr(Merge(*a.Value, *b.Value))}
	}
	return Type{Kind: Unknown}
}
func typePtr(t Type) *Type { return &t }
func FromAST(t *ast.Type) Type {
	if t == nil {
		return Type{Kind: Unknown}
	}
	switch t.Kind {
	case ast.TypeInt:
		return Type{Kind: Int}
	case ast.TypeFloat:
		return Type{Kind: Float}
	case ast.TypeBool:
		return Type{Kind: Bool}
	case ast.TypeString:
		return Type{Kind: String}
	case ast.TypeNull:
		return Type{Kind: Null}
	}
	if t.Element != nil {
		return Type{Kind: Array, Element: typePtr(FromAST(t.Element))}
	}
	if t.Key != nil {
		return Type{Kind: Map, Key: typePtr(FromAST(t.Key)), Value: typePtr(FromAST(t.Value))}
	}
	return Type{Kind: Unknown}
}
func FromValue(v ast.Value) Type {
	switch v.Kind {
	case ast.TypeInt:
		return Type{Kind: Int}
	case ast.TypeFloat:
		return Type{Kind: Float}
	case ast.TypeBool:
		return Type{Kind: Bool}
	case ast.TypeString:
		return Type{Kind: String}
	case ast.TypeNull:
		return Type{Kind: Null}
	case ast.TypeArray:
		e := Type{Kind: Unknown}
		for _, x := range v.Array {
			e = Merge(e, FromValue(x))
		}
		return Type{Kind: Array, Element: typePtr(e)}
	case ast.TypeMap:
		x := Type{Kind: Unknown}
		for _, v := range v.Map {
			x = Merge(x, FromValue(v))
		}
		return Type{Kind: Map, Key: typePtr(Type{Kind: String}), Value: typePtr(x)}
	}
	return Type{Kind: Unknown}
}
