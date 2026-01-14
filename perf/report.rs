const BASELINE_FILE: &'static str = "baseline.txt";
const COMPARE_FILE: &'static str = "compare.txt";

// Baseline and Compare file format
// perf: name | {}
// perf: elapsed(nanos) | {}
// perf: cycles | {}
// perf: exit_code | {}

#[derive(Debug)]
struct RunStat {
    name: String,
    elapsed: std::time::Duration,
    cycles: u64,
    frequency: f64, // MHz
    exit_code: u8,
}

fn main() {
    let baseline_content =
        std::fs::read_to_string(BASELINE_FILE).expect("failed to read baseline file");
    let compare_content =
        std::fs::read_to_string(COMPARE_FILE).expect("failed to read compare file");

    let baseline_stat = process_stat_file(baseline_content);
    let compare_stat = process_stat_file(compare_content);

    for (base, compare) in baseline_stat.iter().zip(compare_stat.iter()) {
        println!("{}", base.name);

        // baseline
        println!(
            "{}",
            line("baseline", format_duration(base.elapsed), base.frequency)
        );

        // compare
        println!(
            "{}",
            line("now", format_duration(compare.elapsed), compare.frequency)
        );

        // time delta
        println!(
            "delta    {}",
            time_delta(base.elapsed.as_nanos(), compare.elapsed.as_nanos())
        );

        println!();
    }
}

fn process_stat_file(content: String) -> Vec<RunStat> {
    let mut stats = vec![];

    // extracts relevant value on each line
    // strips perf: tag
    let lines = content
        .lines()
        .map(|line| {
            line.split("|")
                .skip(1)
                .next()
                .expect("failed to parse file")
                .trim()
        })
        .collect::<Vec<_>>();

    for chunk in lines.chunks_exact(4) {
        let name = chunk[0].to_string();

        let nanos: u128 = chunk[1].parse().expect("failed to parse elapsed time");
        let secs = (nanos / 1_000_000_000) as u64;
        let sub_ns = (nanos % 1_000_000_000) as u32;
        let elapsed = std::time::Duration::new(secs, sub_ns);

        let cycles = chunk[2].parse().expect("failed to parse cycles");
        let exit_code = chunk[3].parse().expect("failed to parse exit code");

        let mhz = cycles as f64 * 1000.0 / nanos as f64;

        stats.push(RunStat {
            name,
            elapsed,
            cycles,
            frequency: mhz,
            exit_code,
        });
    }

    stats
}

fn line(label: &str, time: String, frequency: f64) -> String {
    format!("{label:<8} time: {time:<10} freq: {frequency:>6.2} MHz")
}

fn format_duration(d: std::time::Duration) -> String {
    let ns = d.as_nanos();

    if ns >= 1_000_000_000 {
        format!("{:.3}s", d.as_secs_f64())
    } else if ns >= 1_000_000 {
        format!("{:.3}ms", ns as f64 / 1_000_000.0)
    } else if ns >= 1_000 {
        format!("{:.3}µs", ns as f64 / 1_000.0)
    } else {
        format!("{ns}ns")
    }
}

fn time_delta(base_ns: u128, now_ns: u128) -> String {
    let b = base_ns as f64;
    let n = now_ns as f64;
    let factor = b / n;
    let percent = (factor - 1.0) * 100.0;
    let word = if factor >= 1.0 { "faster" } else { "slower" };
    format!("{:+.1}% ({:.2}x {})", percent, factor, word)
}
