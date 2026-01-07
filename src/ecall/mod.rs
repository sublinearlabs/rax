use crate::{VM, trace::Tracer};

mod halt;
mod constants;

pub fn handle_ecall<T: Tracer>(vm: &mut VM<T>) {
    let func = vm.reg(17);
    
    match func {
        constants::ECALL_HALT => halt::handle_halt(vm),
        _ => panic!("Unknown ecall {}", func),
    }
}