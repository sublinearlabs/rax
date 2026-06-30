use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::aot::{BenchResult, PerfSuite};
use crate::timing::{format_bytes, format_duration_ns};
use crate::PerfResult;

pub fn render_report_files(baseline: &Path, compare: &Path) -> PerfResult<String> {
    let baseline = read_suite(baseline)?;
    let compare = read_suite(compare)?;
    Ok(render_report(&baseline, &compare))
}

fn read_suite(path: &Path) -> PerfResult<PerfSuite> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn render_report(baseline: &PerfSuite, compare: &PerfSuite) -> String {
    let compare_by_name = compare
        .benchmarks
        .iter()
        .map(|bench| (bench.name.as_str(), bench))
        .collect::<BTreeMap<_, _>>();

    let mut out = String::new();
    for base in &baseline.benchmarks {
        let Some(now) = compare_by_name.get(base.name.as_str()) else {
            out.push_str(&format!("{} [missing in compare]\n\n", base.name));
            continue;
        };

        out.push_str(&header(base, now));
        out.push('\n');
        out.push_str(&line_duration(
            "compile",
            base.compile_ns,
            now.compile_ns,
            true,
        ));
        out.push_str(&line_duration(
            "run",
            base.native_run_ns_median,
            now.native_run_ns_median,
            true,
        ));
        out.push_str(&line_duration(
            "min",
            base.native_run_ns_min,
            now.native_run_ns_min,
            true,
        ));
        out.push_str(&line_float(
            "eff",
            base.effective_guest_mhz,
            now.effective_guest_mhz,
            false,
            "MHz",
        ));
        out.push_str(&line_bytes("size", base.output_size, now.output_size));
        out.push_str(&format!(
            "guest    baseline: {:<12} now: {:<12}\n\n",
            base.guest_instructions, now.guest_instructions
        ));
    }

    out
}

fn header(base: &BenchResult, now: &BenchResult) -> String {
    let mut flags = Vec::new();
    if base.jit_exit_code != base.aot_exit_code as u64 {
        flags.push(format!(
            "baseline exit mismatch jit={} aot={}",
            base.jit_exit_code, base.aot_exit_code
        ));
    }
    if now.jit_exit_code != now.aot_exit_code as u64 {
        flags.push(format!(
            "now exit mismatch jit={} aot={}",
            now.jit_exit_code, now.aot_exit_code
        ));
    }
    if !base.stdout_matches || !now.stdout_matches {
        flags.push("stdout mismatch".to_string());
    }
    if !base.stderr_matches || !now.stderr_matches {
        flags.push("stderr mismatch".to_string());
    }

    if flags.is_empty() {
        base.name.clone()
    } else {
        format!("{} [{}]", base.name, flags.join("; "))
    }
}

fn line_duration(label: &str, base: u64, now: u64, lower_is_better: bool) -> String {
    format!(
        "{label:<8} baseline: {:<10} now: {:<10} {}\n",
        format_duration_ns(base),
        format_duration_ns(now),
        delta(base as f64, now as f64, lower_is_better, "faster", "slower")
    )
}

fn line_float(label: &str, base: f64, now: f64, lower_is_better: bool, unit: &str) -> String {
    format!(
        "{label:<8} baseline: {:>8.2} {unit:<3} now: {:>8.2} {unit:<3} {}\n",
        base,
        now,
        delta(base, now, lower_is_better, "faster", "slower")
    )
}

fn line_bytes(label: &str, base: u64, now: u64) -> String {
    format!(
        "{label:<8} baseline: {:<10} now: {:<10} {}\n",
        format_bytes(base),
        format_bytes(now),
        delta(base as f64, now as f64, true, "smaller", "larger")
    )
}

fn delta(base: f64, now: f64, lower_is_better: bool, good: &str, bad: &str) -> String {
    if base == 0.0 || now == 0.0 {
        return "n/a".to_string();
    }

    let factor = if lower_is_better {
        base / now
    } else {
        now / base
    };
    let percent = (factor - 1.0) * 100.0;
    let word = if factor >= 1.0 { good } else { bad };
    format!("{percent:+.1}% ({factor:.2}x {word})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aot::PerfSuite;

    fn bench(name: &str, compile_ns: u64, run_ns: u64, eff: f64) -> BenchResult {
        BenchResult {
            name: name.to_string(),
            elf_path: "bench.elf".to_string(),
            jit_exit_code: 0,
            aot_exit_code: 0,
            guest_instructions: 100,
            compile_ns,
            native_run_ns_median: run_ns,
            native_run_ns_min: run_ns,
            native_run_ns_samples: vec![run_ns],
            effective_guest_mhz: eff,
            output_size: 1024,
            stdout_matches: true,
            stderr_matches: true,
        }
    }

    #[test]
    fn report_matches_benchmarks_by_name() {
        let baseline = PerfSuite {
            kind: "aot".to_string(),
            runs: 1,
            warmups: 0,
            benchmarks: vec![bench("fib", 10, 20, 5.0)],
        };
        let compare = PerfSuite {
            kind: "aot".to_string(),
            runs: 1,
            warmups: 0,
            benchmarks: vec![bench("fib", 5, 10, 10.0)],
        };

        let report = render_report(&baseline, &compare);
        assert!(report.contains("fib"));
        assert!(report.contains("compile"));
        assert!(report.contains("run"));
    }

    #[test]
    fn lower_duration_reports_faster_when_now_is_smaller() {
        assert!(line_duration("run", 100, 50, true).contains("faster"));
        assert!(line_duration("run", 50, 100, true).contains("slower"));
    }
}
