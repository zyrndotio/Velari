package semantic

import "github.com/zyrndotio/Velari/go/ast"

type ExprVisitor interface {
	VisitLiteral(ast.Literal) Type
	VisitVariable(ast.Variable) Type
	VisitArray(ast.Array) Type
	VisitMap(ast.Map) Type
	VisitIndex(ast.Index) Type
	VisitBinary(ast.Binary) Type
	VisitUnary(ast.Unary) Type
	VisitCall(ast.Call) Type
}
type StmtVisitor interface {
	VisitBlock(ast.Block) Flow
	VisitLet(ast.Let) Flow
	VisitAssign(ast.Assign) Flow
	VisitPrint(ast.Print) Flow
	VisitAssert(ast.Assert) Flow
	VisitReturn(ast.Return) Flow
	VisitFunction(ast.Function) Flow
	VisitIf(ast.If) Flow
	VisitRepeat(ast.Repeat) Flow
	VisitWindow(ast.Window) Flow
}
type Flow struct {
	CanFallThrough bool
	CanReturn      bool
}

func WalkExpr(v ExprVisitor, e ast.Expr) Type {
	switch x := e.(type) {
	case ast.Literal:
		return v.VisitLiteral(x)
	case ast.Variable:
		return v.VisitVariable(x)
	case ast.Array:
		return v.VisitArray(x)
	case ast.Map:
		return v.VisitMap(x)
	case ast.Index:
		return v.VisitIndex(x)
	case ast.Binary:
		return v.VisitBinary(x)
	case ast.Unary:
		return v.VisitUnary(x)
	case ast.Call:
		return v.VisitCall(x)
	default:
		return Type{Kind: Unknown}
	}
}
func WalkStmt(v StmtVisitor, s ast.Stmt) Flow {
	switch x := s.(type) {
	case ast.Block:
		return v.VisitBlock(x)
	case ast.Let:
		return v.VisitLet(x)
	case ast.Assign:
		return v.VisitAssign(x)
	case ast.Print:
		return v.VisitPrint(x)
	case ast.Assert:
		return v.VisitAssert(x)
	case ast.Return:
		return v.VisitReturn(x)
	case ast.Function:
		return v.VisitFunction(x)
	case ast.If:
		return v.VisitIf(x)
	case ast.Repeat:
		return v.VisitRepeat(x)
	case ast.Window:
		return v.VisitWindow(x)
	default:
		return Flow{CanFallThrough: true}
	}
}
