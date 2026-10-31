package semantic

import (
	"github.com/zyrndotio/Velari/go/lexer"
	"github.com/zyrndotio/Velari/go/parser"
	"testing"
)

func analyzeSource(t *testing.T, source string) (*Program, []Diagnostic) {
	t.Helper()
	tokens, lexErrors := lexer.New(source).Tokenize()
	if len(lexErrors) > 0 {
		t.Fatalf("lex: %v", lexErrors)
	}
	tree, parseErrors := parser.New(tokens).Parse()
	if len(parseErrors) > 0 {
		t.Fatalf("parse: %v", parseErrors)
	}
	return Analyze(tree)
}
func hasCode(ds []Diagnostic, code string) bool {
	for _, d := range ds {
		if d.Code == code {
			return true
		}
	}
	return false
}
func TestValidTypedProgram(t *testing.T) {
	_, ds := analyzeSource(t, `begin
let count: Int be 3
let values: Array<Int> be [1, 2, 3]
assert length(values) == count
function add(a: Int, b: Int): Int
 return a + b
end
print add(2, 3)
end`)
	if len(ds) != 0 {
		t.Fatalf("unexpected diagnostics: %v", ds)
	}
}
func TestTypeMismatchAndUndefinedVariable(t *testing.T) {
	_, ds := analyzeSource(t, "begin\nlet x: Int be \"wrong\"\nprint missing\nend")
	if !hasCode(ds, "E3003") || !hasCode(ds, "E3001") {
		t.Fatalf("expected type and name diagnostics, got %v", ds)
	}
}
func TestAllPathReturnAnalysis(t *testing.T) {
	_, ds := analyzeSource(t, "begin\nfunction choose(flag: Bool): Int\n if flag then\n  return 1\n otherwise\n  return 2\n end\nend\nend")
	if hasCode(ds, "E3010") {
		t.Fatalf("complete if should return on all paths: %v", ds)
	}
	_, ds = analyzeSource(t, "begin\nfunction choose(flag: Bool): Int\n if flag then\n  return 1\n end\nend\nend")
	if !hasCode(ds, "E3010") {
		t.Fatal("missing return was not reported")
	}
}
func TestVisitorWalksExpressionTypes(t *testing.T) {
	program, ds := analyzeSource(t, "begin\nlet x be 1 + 2.5\nend")
	if len(ds) != 0 {
		t.Fatalf("unexpected diagnostics: %v", ds)
	}
	if len(program.ExprTypes) == 0 {
		t.Fatal("visitor did not record expression types")
	}
}
