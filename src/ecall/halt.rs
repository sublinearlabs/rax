use crate::{trace::Tracer, VM};

pub fn handle_halt<T: Tracer>(vm: &mut VM<T>) {
    vm.halted = true;
    vm.exit_code = vm.reg(10);
}
