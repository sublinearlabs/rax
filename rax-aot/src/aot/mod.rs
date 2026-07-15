mod classification;
pub mod compiler;
mod emission;
mod instruction_context;
mod lazy_context;
mod register_mapping;
mod registers;
mod temp_alloc;
mod translator;

/// Emit x86 instructions through the translator's RefCell-wrapped assembler.
macro_rules! emit_asm {
    ($tr:ident ; $($tt:tt)*) => {{
        let mut __emit_ctx = $tr.emitter.borrow_mut();
        dynasmrt::dynasm!(__emit_ctx ; $($tt)*);
    }};
}
pub(crate) use emit_asm;
