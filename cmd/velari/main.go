package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"

	"github.com/zyrndotio/Velari/go/ast"
	"github.com/zyrndotio/Velari/go/lexer"
	"github.com/zyrndotio/Velari/go/parser"
	velariruntime "github.com/zyrndotio/Velari/go/runtime"
	"github.com/zyrndotio/Velari/go/semantic"
)

var version = "dev"

const projectManifest = `name = "velari-app"
entry = "main.vr"
`

const projectMain = `begin
    print "Hello from VelaRi"
end
`

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
	return velariruntime.New(w).Execute(tree)
}

func repl() {
	in := bufio.NewScanner(os.Stdin)
	fmt.Println("VelaRi REPL — enter quit or exit to leave")
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

func usage(w io.Writer) {
	fmt.Fprintln(w, "VelaRi — readable, high-performance language toolchain")
	fmt.Fprintln(w)
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  velari <command> [options]")
	fmt.Fprintln(w, "  velari --repl")
	fmt.Fprintln(w)
	fmt.Fprintln(w, "Commands:")
	fmt.Fprintln(w, "  version                 Print the installed toolchain version")
	fmt.Fprintln(w, "  help                   Show this help")
	fmt.Fprintln(w, "  check <file>           Lex, parse, and type-check a program")
	fmt.Fprintln(w, "  run <file>             Check and interpret a program")
	fmt.Fprintln(w, "  new <directory>        Create a new VelaRi project")
	fmt.Fprintln(w, "  init [directory]       Initialize a project in a directory")
	fmt.Fprintln(w, "  test [path]            Check and run .vr files under a path")
	fmt.Fprintln(w, "  doctor                 Show installation and platform diagnostics")
	fmt.Fprintln(w)
	fmt.Fprintln(w, "Native application code generation is under active development; use check/run for the current interpreter pipeline.")
}

func writeProject(dir string) error {
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	manifest := filepath.Join(dir, "velari.toml")
	entry := filepath.Join(dir, "main.vr")
	if _, err := os.Stat(manifest); err == nil {
		return fmt.Errorf("project already exists: %s", manifest)
	} else if !os.IsNotExist(err) {
		return err
	}
	if err := os.WriteFile(manifest, []byte(projectManifest), 0o644); err != nil {
		return err
	}
	if err := os.WriteFile(entry, []byte(projectMain), 0o644); err != nil {
		return err
	}
	fmt.Printf("Created VelaRi project in %s\n", dir)
	fmt.Printf("  velari run %s\n", filepath.Join(dir, "main.vr"))
	return nil
}

func initProject(args []string) error {
	dir := "."
	if len(args) > 1 {
		return errors.New("usage: velari init [directory]")
	}
	if len(args) == 1 {
		dir = args[0]
	}
	return writeProject(dir)
}

func testPath(root string) error {
	info, err := os.Stat(root)
	if err != nil {
		return err
	}
	var files []string
	if !info.IsDir() {
		files = []string{root}
	} else {
		err = filepath.Walk(root, func(path string, entry os.FileInfo, walkErr error) error {
			if walkErr != nil {
				return walkErr
			}
			if !entry.IsDir() && strings.HasSuffix(entry.Name(), ".vr") {
				files = append(files, path)
			}
			return nil
		})
		if err != nil {
			return err
		}
	}
	sort.Strings(files)
	if len(files) == 0 {
		return fmt.Errorf("no .vr files found under %s", root)
	}
	failed := 0
	for _, path := range files {
		b, readErr := os.ReadFile(path)
		if readErr == nil {
			readErr = execute(string(b), io.Discard)
		}
		if readErr != nil {
			failed++
			fmt.Printf("FAIL %s: %v\n", path, readErr)
		} else {
			fmt.Printf("PASS %s\n", path)
		}
	}
	if failed > 0 {
		return fmt.Errorf("%d of %d VelaRi programs failed", failed, len(files))
	}
	fmt.Printf("%d VelaRi programs passed\n", len(files))
	return nil
}

func doctor() {
	fmt.Printf("VelaRi version: %s\n", version)
	fmt.Printf("Platform: %s/%s\n", runtime.GOOS, runtime.GOARCH)
	fmt.Printf("Executable: %s\n", executablePath())
	fmt.Printf("Go core: portable lexer, parser, semantic analyzer, and interpreter\n")
	fmt.Printf("Native compiler backend: planned\n")
	fmt.Printf("WebAssembly host adapter: planned\n")
}

func executablePath() string {
	path, err := os.Executable()
	if err != nil {
		return "unknown"
	}
	return path
}

func commandError(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func main() {
	replFlag := flag.Bool("repl", false, "start interactive REPL")
	flag.Usage = func() { usage(os.Stderr) }
	flag.Parse()
	if *replFlag || flag.NArg() == 0 {
		repl()
		return
	}

	command := flag.Arg(0)
	args := flag.Args()[1:]
	switch command {
	case "version", "--version", "-v":
		fmt.Println(version)
	case "help", "--help", "-h":
		usage(os.Stdout)
	case "check":
		if len(args) != 1 {
			commandError(errors.New("usage: velari check <file>"))
		}
		b, err := os.ReadFile(args[0])
		if err == nil {
			_, _, err = compile(string(b))
		}
		commandError(err)
		fmt.Println("ok")
	case "run":
		if len(args) != 1 {
			commandError(errors.New("usage: velari run <file>"))
		}
		b, err := os.ReadFile(args[0])
		if err == nil {
			err = execute(string(b), os.Stdout)
		}
		commandError(err)
	case "new":
		if len(args) != 1 {
			commandError(errors.New("usage: velari new <directory>"))
		}
		commandError(writeProject(args[0]))
	case "init":
		commandError(initProject(args))
	case "test":
		root := "tests"
		if len(args) > 1 {
			commandError(errors.New("usage: velari test [path]"))
		}
		if len(args) == 1 {
			root = args[0]
		}
		commandError(testPath(root))
	case "doctor":
		doctor()
	default:
		usage(os.Stderr)
		os.Exit(2)
	}
}
