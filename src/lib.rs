pub use rax_aot as aot;
pub use rax_core as core;
pub use rax_core::{decode, util};
pub use rax_elfgen as elfgen;
pub use rax_interpreter as interpreter;
#[cfg(feature = "jit")]
pub use rax_jit as jit;

pub use interpreter::init_from_elf;
pub use interpreter::{HostIO, Runner, VM};
