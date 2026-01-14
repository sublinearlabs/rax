use riscv::VM;

pub fn print_perf_stat(vm: &VM, name: &'static str) {
    if std::env::var("PERF").as_deref() == Ok("1") {
        println!("perf: name | {}", name);
        println!("perf: elapsed(nanos) | {}", vm.elapsed.as_nanos());
        println!("perf: cycles | {}", vm.cycles);
        println!("perf: exit_code | {}", vm.exit_code);
    }
}

fn main() {
    println!("shared code not an executable");
}
