use std::process::Command;
use std::path::Path;

fn tayra_bin() -> String {
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    path.push("tayra");
    path.to_string_lossy().to_string()
}

fn create_temp_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    run_git(path, &["init"]);
    run_git(path, &["config", "user.email", "test@test.com"]);
    run_git(path, &["config", "user.name", "Test"]);

    // Initial commit
    std::fs::write(path.join("README.md"), "# Test").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "chore: initial commit"]);

    dir
}

fn run_git(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .expect("failed to run git");
    if !output.status.success() {
        panic!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_no_tags_suggests_0_1_0() {
    let dir = create_temp_repo();
    let path = dir.path();

    // Add a feat commit
    std::fs::write(path.join("lib.rs"), "fn hello() {}").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "feat: add hello function"]);

    let output = Command::new(tayra_bin())
        .args(["--ci", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "v0.1.0");
}

#[test]
fn test_with_tag_patch_bump() {
    let dir = create_temp_repo();
    let path = dir.path();

    // Tag v1.0.0
    run_git(path, &["tag", "v1.0.0"]);

    // Add a fix commit
    std::fs::write(path.join("fix.rs"), "fn fix() {}").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "fix: correct bug"]);

    let output = Command::new(tayra_bin())
        .args(["--ci", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "v1.0.1");
}

#[test]
fn test_with_tag_minor_bump() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "v1.0.0"]);

    std::fs::write(path.join("feat.rs"), "fn feat() {}").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "feat: add new feature"]);

    let output = Command::new(tayra_bin())
        .args(["--ci", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "v1.1.0");
}

#[test]
fn test_with_tag_major_bump() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "v1.0.0"]);

    std::fs::write(path.join("break.rs"), "fn breaking() {}").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "feat!: breaking API change"]);

    let output = Command::new(tayra_bin())
        .args(["--ci", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "v2.0.0");
}

#[test]
fn test_no_prefix_tag() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "1.0.0"]);

    std::fs::write(path.join("fix.rs"), "fn fix() {}").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "fix: bug"]);

    let output = Command::new(tayra_bin())
        .args(["--ci", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "1.0.1");
}

#[test]
fn test_full_output_format() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "v1.2.3"]);

    std::fs::write(path.join("a.rs"), "").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "feat: add auth"]);

    std::fs::write(path.join("b.rs"), "").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "fix: typo"]);

    let output = Command::new(tayra_bin())
        .args(["--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tayra v"));
    assert!(stdout.contains("Current version: v1.2.3"));
    assert!(stdout.contains("minor"));
    assert!(stdout.contains("v1.3.0"));
    assert!(stdout.contains("feat: add auth"));
    assert!(stdout.contains("fix: typo"));
}

#[test]
fn test_create_tag_flag() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "v1.0.0"]);

    std::fs::write(path.join("new.rs"), "").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "feat: new feature"]);

    let output = Command::new(tayra_bin())
        .args(["--tag", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    assert!(output.status.success());

    // Verify the tag was created
    let tag_output = Command::new("git")
        .args(["tag", "-l", "v1.1.0"])
        .current_dir(path)
        .output()
        .unwrap();

    let tags = String::from_utf8_lossy(&tag_output.stdout);
    assert!(tags.contains("v1.1.0"));
}

#[test]
fn test_prefix_override() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "1.0.0"]);

    std::fs::write(path.join("fix.rs"), "").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "fix: bug"]);

    // Override to use v prefix even though existing tag has no prefix
    let output = Command::new(tayra_bin())
        .args(["--ci", "--prefix", "v", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "v1.0.1");
}

#[test]
fn test_not_a_repo_error() {
    let dir = tempfile::tempdir().unwrap();

    let output = Command::new(tayra_bin())
        .args(["--path", dir.path().to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a git repository"));
}

#[test]
fn test_multiple_tags_picks_latest() {
    let dir = create_temp_repo();
    let path = dir.path();

    run_git(path, &["tag", "v1.0.0"]);

    std::fs::write(path.join("a.rs"), "").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "feat: something"]);
    run_git(path, &["tag", "v1.1.0"]);

    std::fs::write(path.join("b.rs"), "").unwrap();
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", "fix: bug after 1.1.0"]);

    let output = Command::new(tayra_bin())
        .args(["--ci", "--path", path.to_str().unwrap()])
        .output()
        .expect("failed to run tayra");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // Should bump from 1.1.0 (latest), not 1.0.0
    assert_eq!(stdout, "v1.1.1");
}
