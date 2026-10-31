package parser

import (
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
	"github.com/zyrndotio/Velari/go/lexer"
)

type Error struct {
	Message string
	Span    ast.Span
}

func (e Error) Error() string {
	return fmt.Sprintf("%s at %d:%d", e.Message, e.Span.Line, e.Span.Column)
}

type Parser struct {
	tokens []lexer.Token
	pos    int
	errors []Error
}

func New(ts []lexer.Token) *Parser { return &Parser{tokens: ts} }
func (p *Parser) peek() lexer.Token {
	if p.pos >= len(p.tokens) {
		return lexer.Token{Kind: lexer.EOF}
	}
	return p.tokens[p.pos]
}
func (p *Parser) advance() lexer.Token {
	t := p.peek()
	if p.pos < len(p.tokens) {
		p.pos++
	}
	return t
}
func (p *Parser) at(k lexer.Kind) bool { return p.peek().Kind == k }
func (p *Parser) eat(k lexer.Kind) bool {
	if p.at(k) {
		p.advance()
		return true
	}
	return false
}
func (p *Parser) need(k lexer.Kind) lexer.Token {
	if p.at(k) {
		return p.advance()
	}
	t := p.peek()
	p.errors = append(p.errors, Error{fmt.Sprintf("expected %q, found %q", k, t.Kind), t.Span})
	return t
}
func (p *Parser) skip() {
	for p.eat(lexer.Newline) {
	}
}
func (p *Parser) Parse() (ast.Stmt, []Error) {
	p.skip()
	begin := p.need(lexer.Begin)
	p.skip()
	body := p.statements(lexer.End)
	p.need(lexer.End)
	p.skip()
	if !p.at(lexer.EOF) {
		p.errors = append(p.errors, Error{"unexpected tokens after program end", p.peek().Span})
	}
	return ast.Block{Statements: body, S: begin.Span}, p.errors
}
func (p *Parser) statements(stop ...lexer.Kind) []ast.Stmt {
	var out []ast.Stmt
	for {
		p.skip()
		done := false
		for _, k := range stop {
			if p.at(k) || p.at(lexer.EOF) {
				done = true
			}
		}
		if done {
			return out
		}
		before := p.pos
		if s := p.statement(); s != nil {
			out = append(out, s)
		}
		if p.pos == before {
			p.advance()
		}
	}
}
func (p *Parser) statement() ast.Stmt {
	t := p.peek()
	switch t.Kind {
	case lexer.Let:
		p.advance()
		n := p.need(lexer.Ident)
		var ty *ast.Type
		if p.eat(lexer.Colon) {
			ty = p.typeExpr()
		}
		p.need(lexer.Be)
		return ast.Let{Name: n.Lexeme, Annotation: ty, Value: p.expr(0), S: t.Span}
	case lexer.Print:
		p.advance()
		return ast.Print{Value: p.expr(0), S: t.Span}
	case lexer.Assert:
		p.advance()
		return ast.Assert{Value: p.expr(0), S: t.Span}
	case lexer.Return:
		p.advance()
		if p.at(lexer.Newline) || p.at(lexer.End) {
			return ast.Return{S: t.Span}
		}
		return ast.Return{Value: p.expr(0), HasValue: true, S: t.Span}
	case lexer.Function:
		return p.function()
	case lexer.If:
		return p.ifStmt()
	case lexer.Repeat:
		p.advance()
		count := p.expr(0)
		p.need(lexer.Times)
		p.skip()
		body := p.statements(lexer.End)
		p.need(lexer.End)
		return ast.Repeat{Count: count, Body: body, S: t.Span}
	case lexer.Create:
		return p.window()
	case lexer.Ident:
		p.advance()
		if p.eat(lexer.Equal) {
			return ast.Assign{Name: t.Lexeme, Value: p.expr(0), S: t.Span}
		}
		p.errors = append(p.errors, Error{"expected assignment after identifier", t.Span})
		return nil
	default:
		p.errors = append(p.errors, Error{fmt.Sprintf("unexpected token %q in statement", t.Kind), t.Span})
		return nil
	}
}
func (p *Parser) function() ast.Stmt {
	t := p.advance()
	n := p.need(lexer.Ident)
	p.need(lexer.LeftParen)
	var ps []ast.Param
	if !p.eat(lexer.RightParen) {
		for {
			pn := p.need(lexer.Ident)
			var ty *ast.Type
			if p.eat(lexer.Colon) {
				ty = p.typeExpr()
			}
			ps = append(ps, ast.Param{Name: pn.Lexeme, Type: ty})
			if p.eat(lexer.RightParen) {
				break
			}
			p.need(lexer.Comma)
		}
	}
	var rt *ast.Type
	if p.eat(lexer.Colon) {
		rt = p.typeExpr()
	}
	p.skip()
	body := p.statements(lexer.End)
	p.need(lexer.End)
	return ast.Function{Name: n.Lexeme, Params: ps, ReturnType: rt, Body: body, S: t.Span}
}
func (p *Parser) ifStmt() ast.Stmt {
	t := p.advance()
	c := p.expr(0)
	p.need(lexer.Then)
	p.skip()
	yes := p.statements(lexer.Otherwise, lexer.End)
	var no []ast.Stmt
	if p.eat(lexer.Otherwise) {
		p.skip()
		no = p.statements(lexer.End)
	}
	p.need(lexer.End)
	return ast.If{Condition: c, Then: yes, Otherwise: no, S: t.Span}
}
func (p *Parser) window() ast.Stmt {
	t := p.advance()
	p.need(lexer.Window)
	if p.at(lexer.Ident) && p.peek().Lexeme == "with" {
		p.advance()
	}
	p.need(lexer.Title)
	title := p.need(lexer.String)
	p.need(lexer.And)
	p.need(lexer.Width)
	w := p.expr(4)
	p.need(lexer.And)
	p.need(lexer.Height)
	h := p.expr(4)
	return ast.Window{Title: title.Lexeme, Width: w, Height: h, S: t.Span}
}
func (p *Parser) typeExpr() *ast.Type {
	t := p.need(lexer.Ident)
	ty := &ast.Type{}
	switch t.Lexeme {
	case "Int":
		ty.Kind = ast.TypeInt
	case "Float":
		ty.Kind = ast.TypeFloat
	case "Bool":
		ty.Kind = ast.TypeBool
	case "String":
		ty.Kind = ast.TypeString
	case "Null":
		ty.Kind = ast.TypeNull
	case "Array":
		ty.Kind = ast.TypeArray
		p.need(lexer.Less)
		ty.Element = p.typeExpr()
		p.need(lexer.Greater)
	case "Map":
		ty.Kind = ast.TypeMap
		p.need(lexer.Less)
		ty.Key = p.typeExpr()
		p.need(lexer.Comma)
		ty.Value = p.typeExpr()
		p.need(lexer.Greater)
	default:
		ty.Kind = ast.TypeUnknown
	}
	return ty
}

var prec = map[lexer.Kind]int{lexer.Or: 1, lexer.And: 2, lexer.EqualEqual: 3, lexer.Plus: 4, lexer.Minus: 4, lexer.Star: 5, lexer.Slash: 5}

func (p *Parser) expr(min int) ast.Expr {
	left := p.prefix()
	for {
		op, ok := prec[p.peek().Kind]
		if !ok || op < min {
			break
		}
		t := p.advance()
		right := p.expr(op + 1)
		mapped := map[lexer.Kind]ast.Op{lexer.Plus: ast.Add, lexer.Minus: ast.Sub, lexer.Star: ast.Mul, lexer.Slash: ast.Div, lexer.EqualEqual: ast.Equal, lexer.And: ast.And, lexer.Or: ast.Or}[t.Kind]
		left = ast.Binary{Left: left, Op: mapped, Right: right, S: t.Span}
	}
	return left
}
func (p *Parser) prefix() ast.Expr {
	t := p.advance()
	var e ast.Expr
	switch t.Kind {
	case lexer.Number:
		k := ast.TypeFloat
		if t.Number == float64(int64(t.Number)) {
			k = ast.TypeInt
		}
		e = ast.Literal{V: ast.Value{Kind: k, Number: t.Number}, S: t.Span}
	case lexer.String:
		e = ast.Literal{V: ast.Value{Kind: ast.TypeString, String: t.Lexeme}, S: t.Span}
	case lexer.True:
		e = ast.Literal{V: ast.Value{Kind: ast.TypeBool, Bool: true}, S: t.Span}
	case lexer.False:
		e = ast.Literal{V: ast.Value{Kind: ast.TypeBool, Bool: false}, S: t.Span}
	case lexer.Null:
		e = ast.Literal{V: ast.Value{Kind: ast.TypeNull}, S: t.Span}
	case lexer.Ident:
		if p.eat(lexer.LeftParen) {
			var args []ast.Expr
			if !p.eat(lexer.RightParen) {
				for {
					args = append(args, p.expr(0))
					if p.eat(lexer.RightParen) {
						break
					}
					p.need(lexer.Comma)
				}
			}
			e = ast.Call{Name: t.Lexeme, Args: args, S: t.Span}
		} else {
			e = ast.Variable{Name: t.Lexeme, S: t.Span}
		}
	case lexer.Minus:
		e = ast.Unary{Op: ast.Neg, Value: p.expr(6), S: t.Span}
	case lexer.LeftParen:
		e = p.expr(0)
		p.need(lexer.RightParen)
	case lexer.LeftBracket:
		var xs []ast.Expr
		if !p.eat(lexer.RightBracket) {
			for {
				xs = append(xs, p.expr(0))
				if p.eat(lexer.RightBracket) {
					break
				}
				p.need(lexer.Comma)
			}
		}
		e = ast.Array{Elements: xs, S: t.Span}
	case lexer.LeftBrace:
		var es []ast.MapEntry
		if !p.eat(lexer.RightBrace) {
			for {
				key := p.need(lexer.String)
				p.need(lexer.Colon)
				es = append(es, ast.MapEntry{Key: key.Lexeme, Value: p.expr(0)})
				if p.eat(lexer.RightBrace) {
					break
				}
				p.need(lexer.Comma)
			}
		}
		e = ast.Map{Entries: es, S: t.Span}
	default:
		p.errors = append(p.errors, Error{"expected expression", t.Span})
		e = ast.Literal{V: ast.Value{Kind: ast.TypeNull}, S: t.Span}
	}
	for p.eat(lexer.LeftBracket) {
		key := p.expr(0)
		r := p.need(lexer.RightBracket)
		e = ast.Index{Target: e, Key: key, S: r.Span}
	}
	return e
}
