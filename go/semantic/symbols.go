package semantic

import "github.com/zyrndotio/Velari/go/ast"

type Binding struct {
	Name    string
	Type    Type
	Mutable bool
	Span    ast.Span
}
type Scope struct{ Bindings map[string]Binding }
type SymbolTable struct{ scopes []Scope }

func NewSymbolTable() *SymbolTable {
	return &SymbolTable{scopes: []Scope{{Bindings: map[string]Binding{}}}}
}
func (s *SymbolTable) Push() { s.scopes = append(s.scopes, Scope{Bindings: map[string]Binding{}}) }
func (s *SymbolTable) Pop() {
	if len(s.scopes) > 1 {
		s.scopes = s.scopes[:len(s.scopes)-1]
	}
}
func (s *SymbolTable) Declare(b Binding) bool {
	scope := &s.scopes[len(s.scopes)-1]
	if _, ok := scope.Bindings[b.Name]; ok {
		return false
	}
	scope.Bindings[b.Name] = b
	return true
}
func (s *SymbolTable) Lookup(name string) (Binding, bool) {
	for n := len(s.scopes) - 1; n >= 0; n-- {
		if b, ok := s.scopes[n].Bindings[name]; ok {
			return b, true
		}
	}
	return Binding{}, false
}

type FunctionSig struct {
	Name    string
	Params  []Type
	Result  Type
	Span    ast.Span
	Builtin bool
}
