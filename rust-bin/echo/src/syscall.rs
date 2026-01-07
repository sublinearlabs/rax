use core::arch::asm;

const SYS_READ: usize = 63;
const SYS_WRITE: usize = 64;

pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;

#[inline(always)]
pub unsafe fn syscall(sys_num: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret: usize;
    asm!(
        "ecall",
        in("a7") sys_num, // Syscall number in a7
        in("a0") arg0,    // FD
        in("a1") arg1,    // Buffer Ptr
        in("a2") arg2,    // Length
        lateout("a0") ret,
        options(nostack)
    );
    ret
}

pub fn sys_read(fd: usize, buf: &mut [u8]) -> usize {
    unsafe { syscall(SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn sys_write(fd: usize, buf: &[u8]) -> usize {
    unsafe { syscall(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len()) }
}