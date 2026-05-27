use std::rc::Rc;

use dynasmrt::{dynasm, DynasmApi};

use crate::aot::{
    register_mapping::{MapTarget, XmmLane},
    registers::X86Gpr,
    temp_alloc::AllocatedTemp,
    translator::Translator,
};

/// Canonical location for a value currently usable as a GPR source.
///
/// Values may already reside in a mapped x86 register or be materialized into
/// a temporary register managed by `TempAllocator`.
#[derive(Clone)]
pub(super) enum ValueLoc<'a> {
    ConstZero,
    Mapped(X86Gpr),
    Temp(Rc<AllocatedTemp<'a>>),
}

impl<'a> ValueLoc<'a> {
    /// Returns the x86 GPR backing this value location.
    ///
    /// # Panics
    ///
    /// Panics for `ConstZero`, which has no backing x86 GPR.
    /// Zero-valued sources must be handled by lowering logic before requesting
    /// a concrete carrier register.
    fn gpr(&self) -> X86Gpr {
        match self {
            Self::ConstZero => panic!("should not materialize const a zero input"),
            Self::Mapped(reg) => *reg,
            Self::Temp(reg) => **reg.as_ref(),
        }
    }
}

/// Prepared input ready for instruction emission.
///
/// Inputs may be materialized from mapped GPRs/XMM lanes or represented as
/// `ConstZero` when the architectural source is `x0`.
#[derive(Clone)]
pub(super) struct PreparedInput<'a> {
    pub(super) src: ValueLoc<'a>,
}

impl<'a> PreparedInput<'a> {
    /// Returns whether this prepared input represents architectural zero (`x0`).
    ///
    /// Callers should use this guard before requesting a concrete carrier via
    /// `gpr()`/`id()`.
    pub(super) fn is_zero(&self) -> bool {
        matches!(self.src, ValueLoc::ConstZero)
    }

    /// Returns the x86 GPR to use as the emitted instruction source.
    ///
    /// # Panics
    ///
    /// Panics when this input is `ConstZero`.
    /// Callers must branch on zero-valued inputs before calling this method.
    pub(super) fn gpr(&self) -> X86Gpr {
        self.src.gpr()
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared input.
    ///
    /// This is the source carrier register code used by instruction encoders.
    /// It is not a RISC-V register index.
    ///
    /// # Panics
    ///
    /// Panics when this input is `ConstZero`.
    /// Callers must branch on zero-valued inputs before calling this method.
    pub(super) fn id(&self) -> u8 {
        self.gpr().id()
    }
}

/// Prepared architectural destination bound to a computed source value.
///
/// A prepared output must be explicitly written back; dropping one without
/// calling `write_back` is considered a programmer error and panics.
pub(super) struct PreparedOutput<'a> {
    src: ValueLoc<'a>,
    dest: MapTarget,
    written_back: bool,
}

impl<'a> PreparedOutput<'a> {
    pub(super) fn new(src: ValueLoc<'a>, dest: MapTarget) -> Self {
        Self {
            src,
            dest,
            written_back: false,
        }
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared output
    /// source carrier.
    ///
    /// This is the source register code used by instruction encoders. It is
    /// not a destination map id and not a RISC-V register index.
    ///
    /// # Panics
    ///
    /// Pancis when this output is `ConstZero`
    pub(super) fn id(&self) -> u8 {
        // panic if zero output
        if self.is_zero() {
            panic!("PreparedOutput::id called on zero/elided output");
        }

        self.src.gpr().id()
    }

    /// Returns whether this prepared output targets architectural zero (`x0`)
    ///
    /// A zero output represents an elided architectural write destination and
    /// does not permit carrier-id lookup or write-back emission.
    fn is_zero(&self) -> bool {
        matches!(self.dest, MapTarget::ConstZero)
    }

    /// Writes a computed source value back to its architectural destination.
    ///
    /// # Contract
    ///
    /// Must be called exactly once for each prepared output.
    ///
    /// Destination semantics are determined by `MapTarget`:
    /// - `ConstZero`: destination write is elided
    /// - `Gpr`: source is written to mapped x86 GPR
    /// - `XmmShared`: source is written to selected shared XMM lane
    /// - `XmmExclusive`: source is written to exclusive XMM destination
    pub(super) fn write_back(mut self, translator: &mut Translator) {
        let src = self.id();
        match self.dest {
            MapTarget::ConstZero => {
                unreachable!(
                    "write_back invariant violated: ConstZero destination should never reach PreparedOutput"
                )
            }
            MapTarget::Gpr(dst) => {
                if self.src.gpr() != dst {
                    dynasm!(translator.emitter
                        ; mov Rq(dst.id()), Rq(src)
                    );
                }
            }
            MapTarget::XmmExclusive(reg) => {
                dynasm!(translator.emitter
                    ; movq Rx(reg.id()), Rq(src)
                );
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::Low,
            } => {
                // Use PINSRQ for shared-low writes to preserve the high 64-bit lane.
                // MOVQ xmm, r64 would clobber/zero the other lane and corrupt its paired shared value.
                dynasm!(translator.emitter
                    ; pinsrq Rx(reg.id()), Rq(src), 0
                );
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::High,
            } => {
                dynasm!(translator.emitter
                    ; pinsrq Rx(reg.id()), Rq(src), 1
                );
            }
        }
        self.written_back = true;
    }
}

impl<'a> Drop for PreparedOutput<'a> {
    /// Enforces strict write-back completion before output teardown.
    fn drop(&mut self) {
        if !self.written_back {
            panic!("PreparedOutput dropped before write_back");
        }
    }
}
