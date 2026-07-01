use rax::init_from_elf;
use rax_jit::Runner;

const FIB_BINARY: &str = "test-bin/rust-bin/fib/fib-ima";

fn main() {
    let mut vm = init_from_elf(FIB_BINARY.to_string());
    let mut runner = Runner::new();
    runner.run(&mut vm);
    assert_eq!(vm.exit_code, 0);
}
