use std::process::Command;

fn main() {
    let mut git_branch = String::from("Unknown");
    let mut git_commit = String::from("Unknown");
    let mut commit_date = String::from("Unknown");
    let mut build_time = String::from("Unknown");

    let branch = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    if branch.is_ok() {
        git_branch = String::from_utf8(branch.unwrap().stdout).unwrap_or("Unknown".to_string());
    }

    let last = Command::new("git").args(&["rev-parse", "HEAD"]).output();
    if last.is_ok() {
        git_commit = String::from_utf8(last.unwrap().stdout).unwrap_or("Unknown".to_string());
    }

    let time = Command::new("git")
        .args(&["log", "-1", "--date=iso", "--pretty=format:%cd"])
        .output();
    if time.is_ok() {
        commit_date = String::from_utf8(time.unwrap().stdout).unwrap_or("Unknown".to_string());
    }

    let b_time = Command::new("date").args(&["+%Y-%m-%d %T %z"]).output();
    if b_time.is_ok() {
        build_time = String::from_utf8(b_time.unwrap().stdout).unwrap_or("Unknown".to_string());
    }

    println!("cargo:rustc-env=BUILD_GIT_BRANCH={}", git_branch);
    println!("cargo:rustc-env=BUILD_GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=BUILD_GIT_DATE={}", commit_date);
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
}
