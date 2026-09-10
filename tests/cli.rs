use std::{fs, process::Command};

fn velari() -> Command {
    Command::new(env!("CARGO_BIN_EXE_velari"))
}

#[test]
fn version_command_reports_current_release() {
    let output = velari().arg("version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "velari 0.4.5-beta.3"
    );
}

#[test]
fn check_and_run_accept_a_source_file() {
    let dir = tempfile_dir("source");
    let file = dir.join("main.vr");
    fs::write(&file, "begin\n print 2 + 3\nend\n").unwrap();
    let checked = velari()
        .args(["check", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(checked.status.success());
    let run = velari()
        .args(["run", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(run.status.success());
    assert!(String::from_utf8_lossy(&run.stdout).contains('5'));
}

#[test]
fn formatter_normalizes_indentation_and_supports_write() {
    let dir = tempfile_dir("fmt");
    let file = dir.join("main.vr");
    fs::write(&file, "begin\nprint 1\nif true then\nprint 2\nend\nend\n").unwrap();
    let output = velari()
        .args(["fmt", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("    print 1"));
    let written = velari()
        .args(["fmt", "--write", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(written.status.success());
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "begin\n    print 1\n    if true then\n        print 2\n    end\nend\n"
    );
    let checked = velari()
        .args(["fmt", "--check", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(checked.status.success());
}

#[test]
fn formatter_check_rejects_unformatted_source() {
    let dir = tempfile_dir("fmt-check");
    let file = dir.join("main.vr");
    fs::write(&file, "begin\nprint 1\nend\n").unwrap();
    let output = velari()
        .args(["fmt", "--check", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("fmt --write"));
}

#[test]
fn manifest_requires_package_section() {
    let dir = tempfile_dir("manifest");
    fs::write(dir.join("vela.toml"), "[desktop]\nbackend = \"auto\"\n").unwrap();
    let output = velari()
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("[package] section"));
}

#[test]
fn manifest_rejects_duplicate_fields() {
    let dir = tempfile_dir("duplicate-manifest");
    fs::write(
        dir.join("vela.toml"),
        "[package]\nname = \"fixture\"\nname = \"again\"\n",
    )
    .unwrap();
    let output = velari()
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate manifest field"));
}

#[test]
fn new_project_is_discoverable_from_inside_the_project() {
    let dir = tempfile_dir("project");
    let project = dir.join("named-project");
    let created = velari()
        .args(["new", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(created.status.success());
    let checked = velari()
        .current_dir(&project)
        .arg("check")
        .output()
        .unwrap();
    assert!(checked.status.success());
    let manifest = fs::read_to_string(project.join("vela.toml")).unwrap();
    assert!(manifest.contains("name = \"named-project\""));
}

#[test]
fn test_command_runs_vr_files_in_tests_directory() {
    let dir = tempfile_dir("tests");
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join("vela.toml"), "[package]\nname = \"fixture\"\n").unwrap();
    fs::write(dir.join("tests/smoke.vr"), "begin\n print 1\nend\n").unwrap();
    let output = velari()
        .args(["test", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 VelaRi test(s) passed"));
}

fn tempfile_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("velari-0-4-5-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn check_reports_stable_diagnostic_context() {
    let dir = tempfile_dir("diagnostic");
    let file = dir.join("bad.vr");
    fs::write(&file, "begin\n print missing\nend\n").unwrap();
    let output = velari()
        .args(["check", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[E3001]"));
    assert!(stderr.contains("variable `missing` used before initialization"));
    assert!(stderr.contains("help:"));
    assert!(stderr.contains("-->"));
}

#[test]
fn system_command_reports_platform() {
    let output = velari().arg("system").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("os:"));
    assert!(text.contains("arch:"));
}

#[test]
fn new_project_has_desktop_metadata_and_package_validation() {
    let dir = tempfile_dir("desktop");
    let project = dir.join("desktop-app");
    assert!(velari()
        .args(["new", project.to_str().unwrap()])
        .output()
        .unwrap()
        .status
        .success());
    let manifest = fs::read_to_string(project.join("vela.toml")).unwrap();
    assert!(manifest.contains("[desktop]"));
    let output = velari()
        .args(["package", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(project.join("target/package/manifest.txt").is_file());
}

#[test]
fn typed_declaration_and_ir_command_work() {
    let dir = tempfile_dir("typed");
    let file = dir.join("typed.vr");
    fs::write(&file, "begin\n let count: Int be 3\n print count\nend\n").unwrap();
    let checked = velari()
        .args(["check", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(checked.status.success());
    let ir = velari()
        .args(["ir", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(ir.status.success());
    assert!(String::from_utf8_lossy(&ir.stdout).contains("function main"));
}

#[test]
fn where_command_reports_user_installation_path() {
    let output = velari().arg("where").output().unwrap();
    assert!(output.status.success());
    let path = String::from_utf8_lossy(&output.stdout);
    assert!(path.contains(".velari"));
    assert!(path.trim_end().ends_with("velari") || path.trim_end().ends_with("velari.exe"));
}
