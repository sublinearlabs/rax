use std::fs;

use riscv::{
    VM,
    trace::{FullTracer, NoopTracer},
};

/// VM with no tracing (zero overhead)
pub type FastVM = VM<NoopTracer>;
/// VM with full execution tracing
pub type TracingVM = VM<FullTracer>;

fn run_test_elf(path: String) {
    println!("running test: {path}");

    let mut vm = VM::<NoopTracer>::init_from_elf(path);
    vm.run();

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

#[test]
fn test_rv64um() {
    let _ = fs::read_dir("test-bin/rv64um")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[test]
fn test_rv64ua() {
    let _ = fs::read_dir("test-bin/rv64ua")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[test]
fn test_rv64uf() {
    let _ = fs::read_dir("test-bin/rv64uf")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}

#[test]
fn test_rv64uc() {
    let _ = fs::read_dir("test-bin/rv64uc")
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
}
