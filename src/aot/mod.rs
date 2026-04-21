use std::fs;

use crate::elf::parse_elf;

pub(crate) mod compiler;
pub(crate) mod register_mapping;

/// Generate an equivalent x86 ELF file given the path to a riscv elf file
///
/// RISCV ELF Constraints
/// - uncompressed format (without the c extension)
/// - single code segment
fn compile_elf(path: &'static str) {
    let bytes = fs::read(path).unwrap();
    let mut elf = parse_elf(&bytes);

    let mut no_executable = 0;

    // find and decode code segments
    for segment in &mut elf.segments {
        if segment.is_executable {
            // ensure that only a single code segment exists in the binary
            // TODO: figure out how to get rid of this restriction
            no_executable += 1;
            if no_executable > 1 {
                panic!("compiler only works with elfs that have a single code segment");
            }

            segment.decode();

            // create a new compiler
            //  elf.global_entry will serve as pc base
            // compile the segements instructions
            // do a by hand assembly comparison
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
