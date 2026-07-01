use std::process::Command;

use crate::PerfResult;

pub fn current_branch() -> PerfResult<String> {
    output("git", &["rev-parse", "--abbrev-ref", "HEAD"])
}

pub fn working_tree_is_clean() -> PerfResult<bool> {
    Ok(output("git", &["status", "--porcelain"])?.is_empty())
}

pub fn checkout(branch: &str) -> PerfResult<()> {
    let status = Command::new("git")
        .args(["checkout", "-q", branch])
        .status()?;
    if !status.success() {
        return Err(format!("failed to checkout `{branch}`").into());
    }
    Ok(())
}

pub struct RestoreBranch {
    branch: String,
}

impl RestoreBranch {
    pub fn new(branch: String) -> Self {
        Self { branch }
    }
}

impl Drop for RestoreBranch {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .args(["checkout", "-q", &self.branch])
            .status();
    }
}

fn output(program: &str, args: &[&str]) -> PerfResult<String> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        return Err(format!("command failed: {program} {}", args.join(" ")).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
