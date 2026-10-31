package main

import (
	"bufio"
	"flag"
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
	"github.com/zyrndotio/Velari/go/lexer"
	"github.com/zyrndotio/Velari/go/parser"
	"github.com/zyrndotio/Velari/go/runtime"
	"io"
	"os"
	"strings"
)

const version = "0.5.0-go"

func compile(source string) (ast.Stmt, error) {
	ts, le := lexer.New(source).Tokenize()
	if len(le) > 0 {
		return nil, le[0]
	}
	tree, pe := parser.New(ts).Parse()
	if len(pe) > 0 {
		return nil, pe[0]
	}
	return tree, nil
}
func execute(source string, w io.Writer) error {
	tree, e := compile(source)
	if e != nil {
		return e
	}
	return runtime.New(w).Execute(tree)
}
func repl() {
	in := bufio.NewScanner(os.Stdin)
	fmt.Print("velari> ")
	for in.Scan() {
		line := strings.TrimSpace(in.Text())
		if line == "quit" || line == "exit" {
			return
		}
		if line != "" {
			if e := execute("begin\n"+line+"\nend", os.Stdout); e != nil {
				fmt.Fprintln(os.Stderr, e)
			}
		}
		fmt.Print("velari> ")
	}
}
func main() {
	replFlag := flag.Bool("repl", false, "start interactive REPL")
	flag.Parse()
	if *replFlag || flag.NArg() == 0 {
		repl()
		return
	}
	switch flag.Arg(0) {
	case "version":
		fmt.Println(version)
	case "check":
		if flag.NArg() < 2 {
			fmt.Fprintln(os.Stderr, "usage: velari check <file>")
			os.Exit(2)
		}
		b, e := os.ReadFile(flag.Arg(1))
		if e == nil {
			_, e = compile(string(b))
		}
		if e != nil {
			fmt.Fprintln(os.Stderr, e)
			os.Exit(1)
		}
		fmt.Println("ok")
	case "run":
		if flag.NArg() < 2 {
			fmt.Fprintln(os.Stderr, "usage: velari run <file>")
			os.Exit(2)
		}
		b, e := os.ReadFile(flag.Arg(1))
		if e == nil {
			e = execute(string(b), os.Stdout)
		}
		if e != nil {
			fmt.Fprintln(os.Stderr, e)
			os.Exit(1)
		}
	default:
		fmt.Fprintln(os.Stderr, "usage: velari [--repl] | version | check <file> | run <file>")
		os.Exit(2)
	}
}
