use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::aot::{BenchResult, PerfSuite};
use crate::timing::{format_bytes, format_duration_ns};
use crate::PerfResult;

const LABEL_WIDTH: usize = 8;
const VALUE_WIDTH: usize = 17;

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
        out.push_str(&format!(
            "{:<LABEL_WIDTH$}{:<VALUE_WIDTH$}{:<VALUE_WIDTH$}{}\n",
            "metric", "baseline", "now", "delta"
        ));
        out.push_str(&format!("{}\n", "-".repeat(60)));
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
            "guest/s",
            base.effective_guest_mhz,
            now.effective_guest_mhz,
            false,
        ));
        out.push_str(&line_ratio("x86/rv", base.x86_per_riscv, now.x86_per_riscv));
        out.push_str(&line_count(
            "x86",
            base.x86_static_instructions,
            now.x86_static_instructions,
        ));
        out.push_str(&line_bytes("code", base.x86_code_bytes, now.x86_code_bytes));
        out.push_str(&line_bytes(
            "jtable",
            base.jump_table_bytes,
            now.jump_table_bytes,
        ));
        out.push_str(&line_bytes("size", base.output_size, now.output_size));
        out.push_str(&line_plain(
            "guest",
            format_integer(base.guest_instructions),
            format_integer(now.guest_instructions),
        ));
        out.push('\n');
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
    row(
        label,
        format_duration_ns(base),
        format_duration_ns(now),
        delta(base as f64, now as f64, lower_is_better, "faster", "slower"),
    )
}

fn line_float(label: &str, base: f64, now: f64, lower_is_better: bool) -> String {
    row(
        label,
        format_frequency_mhz(base),
        format_frequency_mhz(now),
        delta(base, now, lower_is_better, "faster", "slower"),
    )
}

fn line_bytes(label: &str, base: u64, now: u64) -> String {
    row(
        label,
        format_bytes(base),
        format_bytes(now),
        delta(base as f64, now as f64, true, "smaller", "larger"),
    )
}

fn line_count(label: &str, base: u64, now: u64) -> String {
    row(
        label,
        format_integer(base),
        format_integer(now),
        delta(base as f64, now as f64, true, "smaller", "larger"),
    )
}

fn line_ratio(label: &str, base: f64, now: f64) -> String {
    row(
        label,
        format!("{base:.2}"),
        format!("{now:.2}"),
        delta(base, now, true, "smaller", "larger"),
    )
}

fn line_plain(label: &str, base: String, now: String) -> String {
    row(label, base, now, String::new())
}

fn row(label: &str, baseline: String, now: String, delta: String) -> String {
    format!("{label:<LABEL_WIDTH$}{baseline:<VALUE_WIDTH$}{now:<VALUE_WIDTH$}{delta}\n")
}

fn format_frequency_mhz(mhz: f64) -> String {
    if mhz >= 1000.0 {
        format!("{:.2} GHz", mhz / 1000.0)
    } else {
        format!("{mhz:.2} MHz")
    }
}

fn format_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }

    out
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
            riscv_static_instructions: 10,
            x86_static_instructions: 50,
            x86_per_riscv: 5.0,
            x86_code_bytes: 100,
            jump_table_bytes: 80,
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
        assert!(report.contains("metric"));
        assert!(report.contains("baseline"));
        assert!(report.contains("delta"));
        assert!(report.contains("compile"));
        assert!(report.contains("run"));
        assert!(report.contains("x86/rv"));
    }

    #[test]
    fn lower_duration_reports_faster_when_now_is_smaller() {
        assert!(line_duration("run", 100, 50, true).contains("faster"));
        assert!(line_duration("run", 50, 100, true).contains("slower"));
    }

    #[test]
    fn frequency_uses_ghz_at_threshold() {
        assert_eq!(format_frequency_mhz(999.0), "999.00 MHz");
        assert_eq!(format_frequency_mhz(1000.0), "1.00 GHz");
        assert_eq!(format_frequency_mhz(5470.0), "5.47 GHz");
    }

    #[test]
    fn integer_formatting_uses_commas() {
        assert_eq!(format_integer(0), "0");
        assert_eq!(format_integer(115), "115");
        assert_eq!(format_integer(72_000_008), "72,000,008");
        assert_eq!(format_integer(2_165_224_904), "2,165,224,904");
    }
}
