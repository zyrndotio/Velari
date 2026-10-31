package semantic

import (
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
)

type Severity uint8

const (
	Error Severity = iota
	Warning
)

type Note struct {
	Message string
	Span    ast.Span
}
type Diagnostic struct {
	Code     string
	Severity Severity
	Message  string
	Span     ast.Span
	Notes    []Note
}

func (d Diagnostic) Error() string {
	return fmt.Sprintf("%s: %s at %d:%d", d.Code, d.Message, d.Span.Line, d.Span.Column)
}
func errorAt(code, msg string, span ast.Span) Diagnostic {
	return Diagnostic{Code: code, Severity: Error, Message: msg, Span: span}
}
