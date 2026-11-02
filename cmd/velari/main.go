package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"github.com/zyrndotio/Velari/go/ast"
	"github.com/zyrndotio/Velari/go/lexer"
	"github.com/zyrndotio/Velari/go/parser"
	"github.com/zyrndotio/Velari/go/runtime"
	"github.com/zyrndotio/Velari/go/semantic"
	"io"
	"os"
	"strings"
)

var version = "dev"

func compile(source string) (ast.Stmt, *semantic.Program, error) {
	ts, lexErrors := lexer.New(source).Tokenize()
	if len(lexErrors) > 0 {
		return nil, nil, lexErrors[0]
	}
	tree, parseErrors := parser.New(ts).Parse()
	if len(parseErrors) > 0 {
		return nil, nil, parseErrors[0]
	}
	typed, diagnostics := semantic.Analyze(tree)
	if len(diagnostics) > 0 {
		messages := make([]string, len(diagnostics))
		for n, d := range diagnostics {
			messages[n] = d.Error()
		}
		return tree, typed, errors.New(strings.Join(messages, "\n"))
	}
	return tree, typed, nil
}
func execute(source string, w io.Writer) error {
	tree, _, err := compile(source)
	if err != nil {
		return err
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
			if err := execute("begin\n"+line+"\nend", os.Stdout); err != nil {
				fmt.Fprintln(os.Stderr, err)
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
			_, _, e = compile(string(b))
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
