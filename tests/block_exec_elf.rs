use std::fs;

use riscv::{Runner, init_from_elf};

fn run_test_exec_block_elf(path: String) {
    println!("running test: {path}");

    let mut vm = init_from_elf(path);
    let input_hex_string = fs::read_to_string("examples/exec-block.input").unwrap();
    let input_hex_string = input_hex_string.trim();
    let bytes = hex::decode(input_hex_string).unwrap();

    let mut runner = Runner::new();
    runner.set_input_stream(bytes);

    runner.run(&mut vm);

    println!("exit_code {}", vm.exit_code);
    assert!(vm.halted);
    if vm.exit_code != 0 {
        println!("failing test {}", vm.exit_code >> 1);
    }
    assert_eq!(vm.exit_code, 0);
}
#[test]
#[ignore]
fn test_rv64_exec_block() {
    let _ = fs::read_dir("test-bin/rust-bin/exec-block")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_exec_block_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}
