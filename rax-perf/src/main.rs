mod aot;
mod benchmarks;
mod git;
mod report;
mod timing;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use clap::{Parser, Subcommand};

type PerfResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(name = "rax-perf")]
#[command(about = "Developer-only RISC-V performance tooling", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Subcommand)]
enum CommandKind {
    /// Run the AOT benchmark suite and write JSON results.
    Aot {
        #[arg(long, value_name = "FILE")]
        output: PathBuf,

        #[arg(long, default_value_t = 7)]
        runs: usize,

        #[arg(long, default_value_t = 1)]
        warmups: usize,
    },

    /// Compare two AOT benchmark JSON files.
    Report {
        #[arg(value_name = "BASELINE")]
        baseline: PathBuf,

        #[arg(value_name = "COMPARE")]
        compare: PathBuf,

        #[arg(long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Checkout a base branch, benchmark it, return to this branch, and report.
    BranchReport {
        #[arg(long, default_value = "main")]
        base: String,

        #[arg(long, default_value_t = 7)]
        runs: usize,

        #[arg(long, default_value_t = 1)]
        warmups: usize,
    },
}

fn main() -> PerfResult<()> {
    let args = Args::parse();

    match args.command {
        CommandKind::Aot {
            output,
            runs,
            warmups,
        } => {
            let suite = aot::run_suite(runs, warmups)?;
            write_json(&output, &suite)?;
        }
        CommandKind::Report {
            baseline,
            compare,
            output,
        } => {
            let rendered = report::render_report_files(&baseline, &compare)?;
            if let Some(output) = output {
                write_text(&output, &rendered)?;
            }
            print!("{rendered}");
        }
        CommandKind::BranchReport {
            base,
            runs,
            warmups,
        } => branch_report(&base, runs, warmups)?,
    }

    Ok(())
}

fn branch_report(base: &str, runs: usize, warmups: usize) -> PerfResult<()> {
    let original_branch = git::current_branch()?;
    if original_branch == base {
        return Err(format!("cannot run perf-aot from the base branch `{base}`").into());
    }
    if !git::working_tree_is_clean()? {
        return Err("working tree is not clean; commit or stash changes before perf-aot".into());
    }

    let perf_dir = PathBuf::from("target/perf");
    fs::create_dir_all(&perf_dir)?;

    let baseline_path = perf_dir.join("aot-baseline.json");
    let compare_path = perf_dir.join("aot-compare.json");
    let report_path = perf_dir.join("aot-report.txt");

    let _restore = git::RestoreBranch::new(original_branch.clone());

    println!("checking out {base} for AOT baseline");
    git::checkout(base)?;
    run_cargo_aot(&baseline_path, runs, warmups)?;

    println!("checking out {original_branch} for AOT comparison");
    git::checkout(&original_branch)?;
    run_cargo_aot(&compare_path, runs, warmups)?;

    let rendered = report::render_report_files(&baseline_path, &compare_path)?;
    write_text(&report_path, &rendered)?;
    print!("{rendered}");

    Ok(())
}

fn run_cargo_aot(output: &PathBuf, runs: usize, warmups: usize) -> PerfResult<()> {
    let status = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--locked",
            "-p",
            "rax-perf",
            "--",
            "aot",
            "--output",
        ])
        .arg(output)
        .args([
            "--runs",
            &runs.to_string(),
            "--warmups",
            &warmups.to_string(),
        ])
        .status()?;

    if !status.success() {
        return Err(format!("AOT benchmark command failed with status {status}").into());
    }

    Ok(())
}

fn write_json(path: &PathBuf, suite: &aot::PerfSuite) -> PerfResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(suite)?;
    fs::write(path, format!("{json}\n"))?;
    Ok(())
}

fn write_text(path: &PathBuf, text: &str) -> PerfResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(())
}
