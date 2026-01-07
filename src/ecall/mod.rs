use crate::{VM, trace::Tracer};

mod halt;
mod constants;
mod stdin;
mod stdout;

pub fn handle_ecall<T: Tracer>(vm: &mut VM<T>) {
    let func = vm.reg(17);
    
    match func {
        constants::ECALL_HALT => halt::handle_halt(vm),
        constants::ECALL_STD_INPUT => stdin::handle_stdin(vm),
        constants::ECALL_STD_OUTPUT => stdout::handle_stdout(vm),
        _ => panic!("Unknown ecall {}", func),
    }
}