use std::fs;

use riscv::{init_from_elf, Runner};

fn run_test_elf(path: String) {
    println!("running test: {path}");

    let mut vm = init_from_elf(path);
    let mut runner = Runner::new();
    runner.run(&mut vm);

    println!("exit_code {}", vm.exit_code);
    assert!(vm.halted);
    if vm.exit_code != 0 {
        println!("failing test {}", vm.exit_code >> 1);
    }
    assert_eq!(vm.exit_code, 0);
}

#[test]
fn test_rv64ui() {
    let _ = fs::read_dir("test-bin/rv64ui")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[cfg(feature = "ext_m")]
#[test]
fn test_rv64um() {
    let _ = fs::read_dir("test-bin/rv64um")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[cfg(feature = "ext_a")]
#[test]
fn test_rv64ua() {
    let _ = fs::read_dir("test-bin/rv64ua")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[cfg(feature = "ext_f")]
#[test]
fn test_rv64uf() {
    let _ = fs::read_dir("test-bin/rv64uf")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[cfg(feature = "ext_d")]
#[test]
fn test_rv64ud() {
    let _ = fs::read_dir("test-bin/rv64ud")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[cfg(feature = "ext_c")]
#[test]
fn test_rv64uc() {
    let _ = fs::read_dir("test-bin/rv64uc")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}
