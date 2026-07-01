use super::super::{HostIO, VM};
use super::constants;

/// @dev this function would heavily be designed following the Linux ABI
pub fn handle_stdin(vm: &mut VM, io: &mut HostIO) {
    // Arguments according to RISC-V calling convention:
    // a0 (x10) = File Descriptor
    // a1 (x11) = Buffer Pointer (Guest Virtual Address)
    // a2 (x12) = Length to read
    let fd = vm.reg(10);
    let guest_ptr = vm.reg(11);
    let len = vm.reg(12);

    if fd != constants::STDIN_FILENO {
        // Return -1 (error) in a0
        vm.reg_mut(10, (-1i64) as u64);
        return;
    }

    match io.read_stdin(len as usize) {
        Ok(bytes) => {
            vm.write_bytes(guest_ptr as usize, &bytes);
            vm.reg_mut(10, bytes.len() as u64);
        }
        Err(_) => {
            vm.reg_mut(10, (-1i64) as u64);
        }
    }
}
