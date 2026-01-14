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

    dbg!(baseline_stat);
    dbg!(compare_stat);

    // now that I have this data, what do I want to do with it?
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

    // TODO: handle the exit code

    for chunk in lines.chunks_exact(4) {
        let name = chunk[0].to_string();
        let nanos: u64 = chunk[1].parse().expect("failed to parse elapsed time");
        let elapsed = std::time::Duration::from_nanos(nanos);
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
