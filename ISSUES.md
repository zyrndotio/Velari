## Audit findings

### 1. Newline tokens are unreachable, so bare `return` cannot be followed by a statement

The lexer checks `c.is_whitespace()` before its dedicated newline branch. Since `\\n` is whitespace, it is consumed without emitting `TokenKind::Newline`. The parser therefore sees the next statement immediately after `return` and tries to parse it as the return expression.

Reproduction:

```velari
begin
function demo()
	return
	print 1
end
print demo()
end
```

`velari check` reports `expected expression, found Print` on `print 1`. This also makes the parser's explicit support for `Return { value: None }` ineffective except immediately before `end` or EOF.

### 2. Conditional-only function returns are accepted and fall through to `null`

Semantic analysis tracks whether any return was seen anywhere in a function, rather than whether every control-flow path returns. A function with a return only in a branch is accepted even when that branch is not taken.

Reproduction:

```velari
begin
function maybe()
	if 1 == 2 then
		return 7
	end
end
print maybe()
end
```

`velari check` succeeds and `velari run` prints `null`. Either conditional return paths need to be modeled, or the language should reject functions that can fall through without an explicit return.

### 3. Strict Clippy validation fails

`cargo clippy --all-targets --all-features -- -D warnings` fails with 12 errors, including `clippy::possible_missing_else` in the compact lexer/parser/semantic/CLI code, `clippy::unused-unit` in semantic analysis, and `clippy::if_same_then_else` in diagnostic rendering. The normal test suite still passes, but a warnings-as-errors CI check would fail.

### 4. Rust formatting check fails

`cargo fmt -- --check` reports formatting differences across the source files. The implementation is heavily compressed into long lines, so the repository does not pass the standard rustfmt check even though it builds and tests successfully.
