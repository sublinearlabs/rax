#![no_std]
#![no_main]

use core::panic::PanicInfo;
use crate::syscall::{STDOUT, sys_write, STDIN, sys_read};

mod syscall;

/// Entry point for the RISC-V program
/// this program reads input from the host terminal and echoes it back to the terminal.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut buffer = [0u8; 128];

    loop {
        let bytes_read = sys_read(STDIN, &mut buffer);
        if bytes_read == 0 {
            break;
        }
        sys_write(STDOUT, &buffer[0..bytes_read]);
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
