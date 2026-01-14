use std::process::Command;

/// Run a shell command panic on failure
fn sh(cmd: &str) {
    let ok = Command::new("bash")
        .args(["-lc", cmd])
        .status()
        .expect("failed to spawn shell")
        .success();
    if !ok {
        panic!("failed: {}", cmd);
    }
}

/// Run a shell command, return the shell output
fn out(cmd: &str) -> String {
    let output = Command::new("bash")
        .args(["-lc", cmd])
        .output()
        .expect("failed to spawn shell");
    if !output.status.success() {
        panic!("failed: {}", cmd);
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// We need to ensure we end up in the original branch
/// if we encounter a panic at any point during the program
/// we take advantage of the `Drop` implementation to restore
/// the branch
/// relies on unwind panics (panic != abort)
struct Restore(String);
impl Drop for Restore {
    fn drop(&mut self) {
        let _ = Command::new("bash")
            .args(["-lc", &format!("git checkout -q {}", self.0)])
            .status();
    }
}

fn main() {
    // we want to automate the following flow
    // - extract the current branch
    // - ensure it is not main
    // - ensure there is no uncommitted changes on the current branch
    // - checkout main
    // - run make baseline
    // - checkout current branch
    // - run make compare
    // - run make report

    // returns the branch name
    let branch = out("git rev-parse --abbrev-ref HEAD");
    if branch == "main" {
        panic!("cannot generate report on main, no baseline");
    }

    // ensure that the branch is clean (nothing to commit)
    // this signals that it is safe to change branches
    // git status --porcelain
    //  returns nothing if the branch is clean
    let branch_clean = out("git status --porcelain");
    if !branch_clean.is_empty() {
        panic!("\n\n working tree is not clean: please commit first\n\n");
    }

    // ensures that we go back to the current branch in the event
    // of a failure
    let _restore = Restore(branch.clone());

    sh("git checkout -q main");
    sh("make baseline");

    sh(&format!("git checkout -q {}", branch));
    sh("make compare");
    sh("make report");
}

// TODO: determine test process
