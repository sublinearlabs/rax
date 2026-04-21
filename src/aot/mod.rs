use std::fs;

use crate::elf::parse_elf;

pub(crate) mod compiler;
pub(crate) mod register_mapping;

fn compile_elf(path: &'static str) {
    let bytes = fs::read(path).unwrap();
    let mut elf = parse_elf(&bytes);
    for segment in &mut elf.segments {
        if segment.is_executable {
            segment.decode();
            dbg!(&segment.insns);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aot::compile_elf;

    #[test]
    fn compile_echo_ima() {
        const ECHO_BINARY: &str = "test-bin/rust-bin/echo/echo-ima";
        compile_elf(ECHO_BINARY);
    }
}
