use std::fs;

use riscv::{init_from_elf, Runner};

fn run_test_echo_elf(path: String) {
    println!("running test: {path}");

    let mut vm = init_from_elf(path);
    let mut runner = Runner::new();
    runner.set_input_stream("Hola Riscv, buenos días".as_bytes().to_vec());
    runner.run(&mut vm);

    println!("exit_code {}", vm.exit_code);
    assert!(vm.halted);
    if vm.exit_code != 0 {
        println!("failing test {}", vm.exit_code >> 1);
    }
    assert_eq!(vm.exit_code, 0);
}
#[test]
fn test_rv64_echo() {
    let _ = fs::read_dir("test-bin/rust-bin/echo")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_echo_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}
