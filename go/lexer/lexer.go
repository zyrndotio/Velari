package lexer

import (
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
	"strconv"
	"unicode"
)

type Kind string

const (
	Begin        Kind = "begin"
	End          Kind = "end"
	Let          Kind = "let"
	Be           Kind = "be"
	If           Kind = "if"
	Then         Kind = "then"
	Otherwise    Kind = "otherwise"
	Print        Kind = "print"
	Assert       Kind = "assert"
	Repeat       Kind = "repeat"
	Times        Kind = "times"
	Create       Kind = "create"
	Window       Kind = "window"
	Title        Kind = "title"
	Width        Kind = "width"
	Height       Kind = "height"
	True         Kind = "true"
	False        Kind = "false"
	Null         Kind = "null"
	Function     Kind = "function"
	Return       Kind = "return"
	And          Kind = "and"
	Or           Kind = "or"
	Ident        Kind = "identifier"
	Number       Kind = "number"
	String       Kind = "string"
	Plus         Kind = "+"
	Minus        Kind = "-"
	Star         Kind = "*"
	Slash        Kind = "/"
	Equal        Kind = "="
	EqualEqual   Kind = "=="
	LeftParen    Kind = "("
	RightParen   Kind = ")"
	LeftBracket  Kind = "["
	RightBracket Kind = "]"
	LeftBrace    Kind = "{"
	RightBrace   Kind = "}"
	Comma        Kind = ","
	Colon        Kind = ":"
	Less         Kind = "<"
	Greater      Kind = ">"
	Newline      Kind = "newline"
	EOF          Kind = "eof"
)

type Token struct {
	Kind   Kind
	Lexeme string
	Number float64
	Span   ast.Span
}
type Error struct {
	Message string
	Span    ast.Span
}

func (e Error) Error() string {
	return fmt.Sprintf("%s at %d:%d", e.Message, e.Span.Line, e.Span.Column)
}

var keywords = map[string]Kind{"begin": Begin, "end": End, "let": Let, "be": Be, "if": If, "then": Then, "otherwise": Otherwise, "print": Print, "assert": Assert, "repeat": Repeat, "times": Times, "create": Create, "window": Window, "title": Title, "width": Width, "height": Height, "true": True, "false": False, "null": Null, "function": Function, "return": Return, "and": And, "or": Or, "is": EqualEqual}

type Lexer struct {
	src                    []rune
	pos, line, col, offset int
}

func New(src string) *Lexer { return &Lexer{src: []rune(src), line: 1, col: 1} }
func (l *Lexer) current() rune {
	if l.pos >= len(l.src) {
		return 0
	}
	return l.src[l.pos]
}
func (l *Lexer) advance() rune {
	c := l.current()
	if c == 0 {
		return 0
	}
	l.pos++
	l.offset++
	if c == '\n' {
		l.line++
		l.col = 1
	} else {
		l.col++
	}
	return c
}
func (l *Lexer) span(s, line, col, off int) ast.Span {
	return ast.Span{Start: off, End: l.offset, Line: line, Column: col}
}
func (l *Lexer) Tokenize() ([]Token, []Error) {
	var out []Token
	var errs []Error
	for l.current() != 0 {
		c := l.current()
		s, ln, cl, off := l.pos, l.line, l.col, l.offset
		if c == '\n' {
			l.advance()
			out = append(out, Token{Kind: Newline, Span: l.span(s, ln, cl, off)})
			continue
		}
		if unicode.IsSpace(c) {
			l.advance()
			continue
		}
		if c == '#' {
			for l.current() != 0 && l.current() != '\n' {
				l.advance()
			}
			continue
		}
		if c == '"' {
			l.advance()
			v := ""
			closed := false
			for l.current() != 0 {
				ch := l.advance()
				if ch == '"' {
					closed = true
					break
				}
				if ch == '\\' {
					switch l.current() {
					case 'n':
						l.advance()
						v += "\n"
					case 't':
						l.advance()
						v += "\t"
					case '"':
						l.advance()
						v += `"`
					case '\\':
						l.advance()
						v += `\`
					default:
						v += `\`
					}
				} else {
					v += string(ch)
				}
			}
			if !closed {
				errs = append(errs, Error{"unterminated string literal", l.span(s, ln, cl, off)})
			} else {
				out = append(out, Token{Kind: String, Lexeme: v, Span: l.span(s, ln, cl, off)})
			}
			continue
		}
		if unicode.IsDigit(c) || (c == '.' && s+1 < len(l.src) && unicode.IsDigit(l.src[s+1])) {
			text := ""
			dots := 0
			for unicode.IsDigit(l.current()) || l.current() == '.' {
				if l.current() == '.' {
					dots++
				}
				text += string(l.advance())
			}
			n, e := strconv.ParseFloat(text, 64)
			if dots > 1 || e != nil {
				errs = append(errs, Error{"invalid number literal", l.span(s, ln, cl, off)})
			} else {
				out = append(out, Token{Kind: Number, Lexeme: text, Number: n, Span: l.span(s, ln, cl, off)})
			}
			continue
		}
		if unicode.IsLetter(c) || c == '_' {
			text := ""
			for unicode.IsLetter(l.current()) || unicode.IsDigit(l.current()) || l.current() == '_' {
				text += string(l.advance())
			}
			k := Ident
			if x, ok := keywords[text]; ok {
				k = x
			}
			out = append(out, Token{Kind: k, Lexeme: text, Span: l.span(s, ln, cl, off)})
			continue
		}
		two := string([]rune{c, func() rune {
			if s+1 < len(l.src) {
				return l.src[s+1]
			}
			return 0
		}()})
		if two == "==" {
			l.advance()
			l.advance()
			out = append(out, Token{Kind: EqualEqual, Lexeme: two, Span: l.span(s, ln, cl, off)})
			continue
		}
		single := map[rune]Kind{'+': Plus, '-': Minus, '*': Star, '/': Slash, '=': Equal, '(': LeftParen, ')': RightParen, '[': LeftBracket, ']': RightBracket, '{': LeftBrace, '}': RightBrace, ',': Comma, ':': Colon, '<': Less, '>': Greater}
		if k, ok := single[c]; ok {
			l.advance()
			out = append(out, Token{Kind: k, Lexeme: string(c), Span: l.span(s, ln, cl, off)})
			continue
		}
		l.advance()
		errs = append(errs, Error{fmt.Sprintf("unexpected character `%c`", c), l.span(s, ln, cl, off)})
	}
	out = append(out, Token{Kind: EOF, Span: ast.Span{Start: l.offset, End: l.offset, Line: l.line, Column: l.col}})
	return out, errs
}
