mod ast;
mod diagnostics;
mod interpreter;
mod ir;
mod lexer;
mod parser;
mod semantic;
use diagnostics::Diagnostic;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};
const VERSION: &str = "0.4.5-beta.2";

fn install_path() -> Result<PathBuf, String> {
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or("could not determine home directory")?;
    Ok(PathBuf::from(home).join(".velari").join(if cfg!(windows) {
        "velari.exe"
    } else {
        "bin/velari"
    }))
}
fn compile(source: &str, filename: &str, run: bool) -> Result<(), String> {
    let tokens = Lexer::new(source).tokenize().map_err(|es| {
        es.into_iter()
            .map(|e| Diagnostic::error("E1001", e.message, e.span).render(source, filename))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let ast = Parser::new(tokens).parse_program().map_err(|es| {
        es.into_iter()
            .map(|e| Diagnostic::error("E2001", e.message, e.span).render(source, filename))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mut sem = SemanticAnalyzer::new();
    sem.analyze(&ast).map_err(|es| {
        es.into_iter()
            .map(|e| Diagnostic::error("E3001", e.message, e.span).render(source, filename))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    if run {
        Interpreter::new().execute(&ast).map_err(|e| e.0)
    } else {
        Ok(())
    }
}
fn manifest_from(start: &Path) -> Option<PathBuf> {
    let mut p = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = p.join("vela.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !p.pop() {
            return None;
        }
    }
}
fn manifest_data(manifest: &Path) -> Result<(String, PathBuf, bool), String> {
    let text = fs::read_to_string(manifest).map_err(|e| e.to_string())?;
    let mut name = "velari-app".to_string();
    let mut entry = None;
    let mut desktop = false;
    let mut package_seen = false;
    let mut seen_keys = HashSet::new();
    let mut section = "package";
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = &line[1..line.len() - 1];
            if section != "package" && section != "desktop" {
                return Err(format!("unsupported manifest section `{section}`"));
            }
            desktop = section == "desktop";
            if section == "package" {
                package_seen = true;
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("invalid vela.toml line {}", line_no + 1));
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        let key_id = format!("{section}.{key}");
        if !seen_keys.insert(key_id) {
            return Err(format!(
                "duplicate manifest field `{key}` on line {}",
                line_no + 1
            ));
        }
        match key {
            "name" => {
                if value.is_empty() {
                    return Err("package name cannot be empty".into());
                }
                if section != "package" {
                    return Err(format!(
                        "`name` is only valid in [package] on line {}",
                        line_no + 1
                    ));
                }
                if !value
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
                {
                    return Err(format!(
                        "package name contains invalid characters on line {}",
                        line_no + 1
                    ));
                }
                name = value.to_string()
            }
            "version" => {
                if section != "package" {
                    return Err(format!(
                        "`version` is only valid in [package] on line {}",
                        line_no + 1
                    ));
                }
                if value.is_empty() {
                    return Err("package version cannot be empty".into());
                }
            }
            "entry" => {
                if section != "package" {
                    return Err(format!(
                        "`entry` is only valid in [package] on line {}",
                        line_no + 1
                    ));
                }
                if value.is_empty() {
                    return Err("entry cannot be empty".into());
                }
                entry = Some(value.to_string())
            }
            "backend" => {
                if section != "desktop" || !matches!(value, "auto" | "windows" | "linux" | "macos")
                {
                    return Err(format!(
                        "unsupported desktop backend `{value}` on line {}",
                        line_no + 1
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "unsupported manifest field `{key}` on line {}",
                    line_no + 1
                ))
            }
        }
    }
    if !package_seen {
        return Err("manifest must contain a [package] section".into());
    }
    let path = manifest
        .parent()
        .unwrap_or(Path::new("."))
        .join(entry.unwrap_or_else(|| "src/main.vr".into()));
    if !path.is_file() {
        return Err(format!("manifest entry does not exist: {}", path.display()));
    }
    Ok((name, path, desktop))
}
fn source_path(arg: Option<String>) -> Result<PathBuf, String> {
    let path = arg
        .map(PathBuf::from)
        .unwrap_or(env::current_dir().map_err(|e| e.to_string())?);
    if path.is_file() {
        Ok(path)
    } else {
        let m = manifest_from(&path).ok_or_else(|| {
            "could not find vela.toml; provide a .vr source file or run inside a project"
                .to_string()
        })?;
        Ok(manifest_data(&m)?.1)
    }
}
fn project_root(arg: Option<String>) -> Result<PathBuf, String> {
    let p = arg
        .map(PathBuf::from)
        .unwrap_or(env::current_dir().map_err(|e| e.to_string())?);
    let m = manifest_from(&p).ok_or_else(|| "could not find vela.toml".to_string())?;
    Ok(m.parent().unwrap_or(Path::new(".")).to_path_buf())
}

fn format_source(source: &str) -> String {
    let mut indent = 0usize;
    let mut output = Vec::new();
    for raw in source.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            if output.last().is_some_and(|line: &String| !line.is_empty()) {
                output.push(String::new());
            }
            continue;
        }
        let first = trimmed.split_whitespace().next();
        if matches!(first, Some("end") | Some("otherwise")) {
            indent = indent.saturating_sub(1);
        }
        output.push(format!("{}{}", "    ".repeat(indent), trimmed));
        if matches!(
            first,
            Some("begin") | Some("function") | Some("if") | Some("otherwise") | Some("repeat")
        ) {
            indent += 1;
        }
    }
    format!("{}\n", output.join("\n"))
}

fn format_file(path: &Path, write: bool) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let formatted = format_source(&source);
    if write {
        fs::write(path, formatted).map_err(|e| e.to_string())?;
        println!("formatted {}", path.display());
    } else {
        print!("{formatted}");
    }
    Ok(())
}

fn check_format_file(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if source == format_source(&source) {
        Ok(())
    } else {
        Err(format!(
            "{} is not formatted; run `velari fmt --write {}`",
            path.display(),
            path.display()
        ))
    }
}

fn run_tests(root: &Path) -> Result<(), String> {
    let dir = root.join("tests");
    if !dir.is_dir() {
        println!("no VelaRi tests found");
        return Ok(());
    }
    let mut total = 0;
    for item in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let path = item.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|x| x.to_str()) != Some("vr") {
            continue;
        }
        total += 1;
        let source = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        compile(&source, path.to_str().unwrap_or("<test>"), true)?;
    }
    println!("{total} VelaRi test(s) passed");
    Ok(())
}
fn install() -> Result<(), String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let dest = install_path()?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let temporary = dest.with_extension("installing");
    fs::copy(exe, &temporary).map_err(|e| e.to_string())?;
    fs::rename(&temporary, &dest).map_err(|e| e.to_string())?;
    println!("installed VelaRi to {}", dest.display());
    println!(
        "add {} to your user PATH to run `velari` globally",
        dest.parent().unwrap_or(Path::new(".")).display()
    );
    Ok(())
}
fn uninstall() -> Result<(), String> {
    let dest = install_path()?;
    if dest.is_file() {
        fs::remove_file(&dest).map_err(|e| e.to_string())?;
        println!("uninstalled VelaRi from {}", dest.display());
    } else {
        println!("VelaRi is not installed at {}", dest.display());
    }
    Ok(())
}
fn where_installed() -> Result<(), String> {
    println!("{}", install_path()?.display());
    Ok(())
}
fn package(root: &Path) -> Result<(), String> {
    let (_, entry, desktop) = manifest_data(&root.join("vela.toml"))?;
    let source = fs::read_to_string(&entry).map_err(|e| e.to_string())?;
    compile(&source, entry.to_str().unwrap_or("<entry>"), false)?;
    let out = root.join("target").join("package");
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut f = fs::File::create(out.join("manifest.txt")).map_err(|e| e.to_string())?;
    writeln!(
        f,
        "entry={}
desktop={}
status=validated; native executable generation is planned for a future release",
        entry.display(),
        desktop
    )
    .map_err(|e| e.to_string())?;
    println!("package validation succeeded: {}", out.display());
    Ok(())
}
fn usage() {
    eprintln!("VelaRi {VERSION}\nusage: velari <check|run|test|build|package|fmt|ir|install|uninstall|where|system|new|version> [file|project]");
}
fn main() {
    let mut args = env::args().skip(1);
    let command = args.next();
    let result = match command.as_deref() {
        Some("version") => {
            println!("velari {VERSION}");
            Ok(())
        }
        Some("ir") => {
            let p = source_path(args.next());
            p.and_then(|p| {
                let source = fs::read_to_string(p).map_err(|e| e.to_string())?;
                let tokens = Lexer::new(&source)
                    .tokenize()
                    .map_err(|e| format!("{} errors", e.len()))?;
                let ast = Parser::new(tokens)
                    .parse_program()
                    .map_err(|_| "parse failed".to_string())?;
                println!("{}", ir::Lowerer::lower(&ast).text());
                Ok(())
            })
        }
        Some("system") => {
            println!("VelaRi {VERSION}");
            println!("os: {}", env::consts::OS);
            println!("arch: {}", env::consts::ARCH);
            println!("family: {}", env::consts::FAMILY);
            Ok(())
        }
        Some("install") => install(),
        Some("uninstall") => uninstall(),
        Some("where") => where_installed(),
        Some("fmt") => {
            let first = args.next();
            let (mode, path) = if first.as_deref() == Some("--write") {
                ("write", args.next())
            } else if first.as_deref() == Some("--check") {
                ("check", args.next())
            } else {
                ("print", first)
            };
            match path {
                Some(path) if mode == "write" => format_file(Path::new(&path), true),
                Some(path) if mode == "check" => check_format_file(Path::new(&path)),
                Some(path) => format_file(Path::new(&path), false),
                None => Err(
                    "fmt requires a source file; use `fmt --check <file>` or `fmt --write <file>`"
                        .into(),
                ),
            }
        }
        Some("package") => project_root(args.next()).and_then(|r| package(&r)),
        Some("check") | Some("run") | Some("build") => {
            let run = command.as_deref() == Some("run");
            let build = command.as_deref() == Some("build");
            source_path(args.next()).and_then(|p|{let source=fs::read_to_string(&p).map_err(|e|e.to_string())?;compile(&source,p.to_str().unwrap_or("<source>"),run).map(|_|{if build{println!("build validation succeeded; use `velari package` for desktop package validation")}else if !run{println!("check succeeded")}})})
        }
        Some("test") => project_root(args.next()).and_then(|r| run_tests(&r)),
        Some("new") => {
            let dir = args.next().unwrap_or_else(|| "velari-app".into());
            let package_name = Path::new(&dir)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("velari-app")
                .replace(' ', "-");
            fs::create_dir_all(Path::new(&dir).join("src")).map_err(|e|e.to_string()).and_then(|_|{fs::create_dir_all(Path::new(&dir).join("tests")).map_err(|e|e.to_string())?;fs::create_dir_all(Path::new(&dir).join("assets")).map_err(|e|e.to_string())?;fs::write(Path::new(&dir).join("src/main.vr"),"begin\n print \"hello, VelaRi\"\nend\n").map_err(|e|e.to_string())?;fs::write(Path::new(&dir).join("vela.toml"),format!("[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nentry = \"src/main.vr\"\n\n[desktop]\nbackend = \"auto\"\n")).map_err(|e|e.to_string())?;println!("created {dir}");Ok(())})
        }
        _ => {
            usage();
            Ok(())
        }
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
