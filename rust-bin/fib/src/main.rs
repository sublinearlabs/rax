#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Entry point for the RISC-V program
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Calculate fibonacci(10_000)
    let n = 9_000_000;
    let n = 10_000;
    let result = fib(n);

    // Store result in a0 register and exit
    // For bare-metal, we'll just loop forever after computation
    // The result can be observed in a debugger or emulator
    unsafe {
        core::arch::asm!(
            "mv a0, {0}",
            in(reg) result,
        );
    }

    // Exit with ecall (for emulators that support it)
    exit(0);
}

/// Iterative Fibonacci implementation
/// Returns the nth Fibonacci number
#[inline(never)]
pub fn fib(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut prev = 0u64;
    let mut curr = 1u64;

    for _ in 2..=n {
        let next = prev.wrapping_add(curr);
        prev = curr;
        curr = next;
    }

    curr
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
