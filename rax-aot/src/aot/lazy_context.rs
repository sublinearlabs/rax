// cache represents the current physical location of a map target
// .id() needs to do a fresh resolution each time
// need a new structure to represent anti clobber register locations
// while still keeping output handling safe

// where do we start, we need the builder

use crate::aot::registers::{RiscvRegister, X86Gpr};

/// Builder for preparing instruction operands before AOT emission.
///
/// The builder collects architectural inputs, an optional architectural output,
/// and any x86 GPRs that the emitted instruction may clobber. `build()`
/// materializes the requested operands and emits any setup moves needed to
/// preserve clobbered mapped values.
pub(super) struct InstructionContextBuilder<const NI: usize, const NCT: usize> {
    /// Architectural source registers consumed by the instruction.
    inputs: Option<[RiscvRegister; NI]>,
    /// Architectural destination register produced by the instruction, if any.
    output: Option<RiscvRegister>,
    /// x86 GPRs that must be preserved across instruction emission.
    clobber_targets: Option<[X86Gpr; NCT]>,
}

impl<const NI: usize, const NCT: usize> InstructionContextBuilder<NI, NCT> {
    /// Creates an empty instruction context builder.
    ///
    /// Inputs, output, and clobber targets may be supplied with the builder
    /// methods before calling `build()`.
    pub(super) fn new() -> Self {
        Self {
            inputs: None,
            output: None,
            clobber_targets: None,
        }
    }

    /// Sets the architectural source registers for this instruction.
    pub(super) fn set_inputs(mut self, inputs: [RiscvRegister; NI]) -> Self {
        self.inputs = Some(inputs);
        self
    }

    /// Sets the architectural destination register for this instruction.
    pub(super) fn set_output(mut self, output: RiscvRegister) -> Self {
        self.output = Some(output);
        self
    }

    /// Marks x86 GPRs that must not be clobbered by instruction emission.
    ///
    /// During `build()`, any live mapped value in these registers is moved to a
    /// temp and a restore output is recorded in the resulting
    /// `InstructionContext`.
    pub(super) fn ensure_no_clobber(mut self, clobber_targets: [X86Gpr; NCT]) -> Self {
        self.clobber_targets = Some(clobber_targets);
        self
    }

    // TODO: add builder comment
    pub(super) fn build(self) {
        todo!()
    }
}
