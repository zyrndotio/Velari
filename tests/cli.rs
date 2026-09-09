use std::{fs, process::Command};

fn velari() -> Command { Command::new(env!("CARGO_BIN_EXE_velari")) }

#[test]
fn version_command_reports_current_release() {
    let output = velari().arg("version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "velari 0.3.5");
}

#[test]
fn check_and_run_accept_a_source_file() {
    let dir = tempfile_dir("source");
    let file = dir.join("main.vr");
    fs::write(&file, "begin\n print 2 + 3\nend\n").unwrap();
    let checked = velari().args(["check", file.to_str().unwrap()]).output().unwrap();
    assert!(checked.status.success());
    let run = velari().args(["run", file.to_str().unwrap()]).output().unwrap();
    assert!(run.status.success());
    assert!(String::from_utf8_lossy(&run.stdout).contains("5"));
}

#[test]
fn new_project_is_discoverable_from_inside_the_project() {
    let dir = tempfile_dir("project");
    let project = dir.join("named-project");
    let created = velari().args(["new", project.to_str().unwrap()]).output().unwrap();
    assert!(created.status.success());
    let checked = velari().current_dir(&project).arg("check").output().unwrap();
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
    let output = velari().args(["test", dir.to_str().unwrap()]).output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 VelaRi test(s) passed"));
}

fn tempfile_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("velari-0-3-2-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn check_reports_stable_diagnostic_context() {
    let dir = tempfile_dir("diagnostic");
    let file = dir.join("bad.vr");
    fs::write(&file, "begin\n print missing\nend\n").unwrap();
    let output = velari().args(["check", file.to_str().unwrap()]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[E3001]"));
    assert!(stderr.contains("variable `missing` used before initialization"));
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
    assert!(velari().args(["new", project.to_str().unwrap()]).output().unwrap().status.success());
    let manifest = std::fs::read_to_string(project.join("vela.toml")).unwrap();
    assert!(manifest.contains("[desktop]"));
    let output = velari().args(["package", project.to_str().unwrap()]).output().unwrap();
    assert!(output.status.success());
    assert!(project.join("target/package/manifest.txt").is_file());
}
