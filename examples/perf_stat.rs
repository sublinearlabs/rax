use riscv::{Runner, VM};

pub fn print_perf_stat(runner: &Runner, vm: &VM, name: &'static str) {
    if std::env::var("PERF").as_deref() == Ok("1") {
        println!("perf: name | {}", name);
        println!("perf: elapsed(nanos) | {}", runner.elapsed().as_nanos());
        println!("perf: cycles | {}", runner.cycles());
        println!("perf: exit_code | {}", vm.exit_code);
    }
}

fn main() {
    println!("shared code not an executable");
}
