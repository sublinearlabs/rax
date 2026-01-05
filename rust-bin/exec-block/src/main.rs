#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::slice;


mod exec;
mod prelude;
mod utils;

extern crate alloc;



/// Base address where input data is stored
/// The first 8 bytes contain the length (as u64), followed by the actual data
const INPUT_BASE_ADDR: usize = 0x80000000;

/// Entry point for the RISC-V program
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Read input length from memory (first 8 bytes at INPUT_BASE_ADDR)
    let input_len = unsafe { core::ptr::read_volatile(INPUT_BASE_ADDR as *const u64) as usize };

    // Read input data from memory (starts after the length field)
    let input_data_addr = INPUT_BASE_ADDR + 8;
    let input: &[u8] = unsafe { slice::from_raw_parts(input_data_addr as *const u8, input_len) };

    // Execute block by calling runner()
    let result: [u8; 32] = exec::runner(input);

    // Convert the 32-byte result into 4 u64 values for registers a0, a1, a2, a3
    // Each register holds 8 bytes (64 bits) on RV64
    let a0 = u64::from_le_bytes(result[0..8].try_into().unwrap());
    let a1 = u64::from_le_bytes(result[8..16].try_into().unwrap());
    let a2 = u64::from_le_bytes(result[16..24].try_into().unwrap());
    let a3 = u64::from_le_bytes(result[24..32].try_into().unwrap());

    // Store result in a0, a1, a2, a3 registers
    unsafe {
        core::arch::asm!(
            "mv a0, {0}",
            "mv a1, {1}",
            "mv a2, {2}",
            "mv a3, {3}",
            in(reg) a0,
            in(reg) a1,
            in(reg) a2,
            in(reg) a3,
        );
    }

    // Exit with ecall (for emulators that support it)
    exit(0);
}

/// Exit the program using RISC-V ecall
fn exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "li a7, 93",    // syscall number for exit
            "mv a0, {0}",   // exit code
            "ecall",
            in(reg) code,
            options(noreturn)
        );
    }
}

/// Panic handler required for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
