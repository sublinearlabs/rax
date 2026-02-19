use crate::{ecall::constants, trace::Tracer, VM};

/// @dev this function would heavily be designed following the Linux ABI
pub fn handle_stdin<T: Tracer>(vm: &mut VM<T>) {
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

    let available_bytes = vm.input_stream.len() - vm.input_cursor;
    let bytes_to_read = std::cmp::min(len as usize, available_bytes);

    let src_slice = &vm.input_stream.clone()[vm.input_cursor..vm.input_cursor + bytes_to_read];
    vm.write_bytes(guest_ptr as usize, src_slice);

    vm.input_cursor += bytes_to_read;

    vm.reg_mut(10, bytes_to_read as u64);
}
