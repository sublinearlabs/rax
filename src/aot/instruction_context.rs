use std::{
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

use dynasmrt::{dynasm, DynasmApi};

use crate::aot::{
    register_mapping::{MapTarget, XmmLane},
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::{AllocatedTemp, TempAllocator},
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

struct InstructionContextBuilder<const NI: usize, const NCT: usize> {
    inputs: Option<[RiscvRegister; NI]>,
    output: Option<RiscvRegister>,
    clobber_targets: Option<[X86Gpr; NCT]>,
}

impl<const NI: usize, const NCT: usize> InstructionContextBuilder<NI, NCT> {
    fn new() -> Self {
        Self {
            inputs: None,
            output: None,
            clobber_targets: None,
        }
    }

    fn set_inputs(mut self, inputs: [RiscvRegister; NI]) -> Self {
        self.inputs = Some(inputs);
        self
    }

    fn set_output(mut self, output: RiscvRegister) -> Self {
        self.output = Some(output);
        self
    }

    fn ensure_no_clobber(mut self, clobber_targets: [X86Gpr; NCT]) -> Self {
        self.clobber_targets = Some(clobber_targets);
        self
    }

    fn build<'a>(
        self,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> InstructionContext<'a, NI, NCT> {
        // this should take all the inputs and create the proper instruction context
        // I'd need to define the goals and constraints that need to be met
        // we have the inputs, we have the outputs
        // if they point to the same thing, then they should have the same value
        // (this includes temps)
        //
        // things become interesting when we consider the clobber sites
        // there is an interaction between the clobber site and the inputs
        // but also between inputs
        //
        // the algorithm I am implementing first will be pretty simple
        // if the data contained in a clobber site is valuable we move that data to a temp
        // if an input points to a clobber site then we remap that input
        // essentially returning something that has to be relocated.
        //
        // this is why I needed a new type to represent relocated and non relocated types
        // as the inputs themselves might get relocated
        // technically all we want is the value that they contain
        //
        // inputs just need a location that the data can be read, but they don't need to be written
        // back
        // but if we clobber to an input location after clobbering is done, we want to ensure that the
        // clobbered location contain the input value
        // so essentially we remap the input, but we also point the clobber status to that point
        // essentially we have to solve for the action to take, very interesting, so the input
        // doesn't need to be relocated.
        //
        // so things that need to be relocated are the output and theh clobber restores
        //
        // now I need to figure out the actual algorithm I'd need for this mapping
        // I think clobber sites put up the most constraints
        //
        // when there is no overlap between clobber sites and the operands
        // we should just copy the clobber sites into temps, with prepared output
        // via emissions
        //
        // what if one of the inputs point there?
        // - this can't happen if the input is in xmm
        // - so the input will have to be in GPR, in that case, we need to create a relocation from
        // that position to a temp
        // but this is exactly the same operation as above
        //
        // hence we don't even need to know if there is an overlap, what we just need to do is keep
        // track of any remapping of gprs to temp and what the new mapping is
        // then for every input / output we check if their normal location has been remapped, if it
        // has we just duplcate the new location (clobber restore will handle writeback)
        //
        // sketch
        // - handle clobber (build remap table)
        // - handle input
        // - handle output
        //
        // xmm handling is pretty easy, for inputs that are xmm and for outputs that are xmm
        // we just use the temps and we are good. things only get interesting when the input /
        // output is not xmm, if that is the case, then no emissions will be needed, we just check
        // for the appropriate mapping
        //
        // inputs and outputs need a way to know if their register has been seen before
        // what is a good information structure for this
        // also the inputs will need to know if they have been remapped and likewise the output
        //
        // we always start with a riscv register location, from this there is a one to one mapping
        // with an x86 target
        // zero to zero, so that is fine
        //
        // let us start with the cache, I have a feeling that this is all that will be required.
        // for that to be the case, it is important for the cache to not have any distingushing
        // information about the input or output.
        // It should represent a mapping from a riscv register / map target to an x86 gpr,
        // which is essentially a ValueLoc.
        // this means the cache can be represented as a vec of 32 ValueLocs.
        //
        // for x86 registers that are clobbered, we pick a temp for them (valueloc) and then we
        // index the cache based on the map target and then insert that location into the cache.
        // for inputs that come from xmm, we do the same thing
        // for inputs that are clobbered, we should already have a cache entry for them
        // for inputs that are just gprs, we duplicate the location within
        // for the output, we just check the cache but no entry to cache is required (we do the
        // output last)
        //
        // once we are done, the cache should contain the minimal amount of emissions required
        // to set things up
        // but is there an ordering? I don't think so, I feel they should all be independent

        let mut cache: HashMap<MapTarget, ValueLoc> = HashMap::new();

        let mut clobber_restore = vec![];
        let mut prepared_inputs = Vec::with_capacity(NI);
        let mut prepared_output = None;

        // handle clobber
        if let Some(clobber_targets) = self.clobber_targets {
            for clobbered_reg in clobber_targets {
                let target = MapTarget::Gpr(clobbered_reg);
                let Entry::Vacant(entry) = cache.entry(target) else {
                    continue;
                };

                let temp_reg = ValueLoc::Temp(Rc::new(temp_allocator.allocate().unwrap()));

                // TODO: put id method for valueloc
                dynasm!(translator.emitter ; mov Rq(clobbered_reg as u8), Rq(temp_reg.gpr().id()));

                entry.insert(temp_reg.clone());

                clobber_restore.push(PreparedOutput::new(temp_reg, target));
            }
        }

        // handle input
        if let Some(inputs) = self.inputs {
            for input in inputs {
                let target = translator.reg_map.get(&input);

                let prepared_input = match cache.entry(*target) {
                    Entry::Occupied(entry) => PreparedInput {
                        src: entry.get().clone(),
                    },
                    Entry::Vacant(entry) => {
                        // here is where we actually handle the input creation
                        // we need to match the target and then produce some prepared input

                        match target {
                            MapTarget::ConstZero => PreparedInput {
                                src: ValueLoc::ConstZero,
                            },
                            MapTarget::Gpr(x86_gpr) => PreparedInput {
                                src: ValueLoc::Mapped(*x86_gpr),
                            },
                            MapTarget::XmmExclusive(reg)
                            | MapTarget::XmmShared {
                                reg,
                                lane: XmmLane::Low,
                            } => {
                                // TODO: handle unwrap here and else where
                                let temp = temp_allocator.allocate().unwrap();
                                dynasm!(translator.emitter ; movq Rq(temp.id()), Rx(reg.id()));
                                let val = ValueLoc::Temp(Rc::new(temp));
                                entry.insert(val.clone());
                                PreparedInput { src: val }
                            }
                            MapTarget::XmmShared { reg, lane } => {
                                let temp = temp_allocator.allocate().unwrap();
                                dynasm!(translator.emitter ; pextrq Rq(temp.id()), Rx(reg.id()), 1);
                                let val = ValueLoc::Temp(Rc::new(temp));
                                entry.insert(val.clone());
                                PreparedInput { src: val }
                            }
                        }
                    }
                };

                prepared_inputs.push(prepared_input);
            }
        }

        // handle output
        if let Some(output) = self.output {
            let target = translator.reg_map.get(&output);

            prepared_output = Some(match cache.entry(*target) {
                Entry::Occupied(entry) => PreparedOutput::new(entry.get().clone(), *target),
                Entry::Vacant(entry) => {
                    let src = match *target {
                        MapTarget::ConstZero => {
                            panic!("prepare_output invariant violated: x0/ConstZero destination must be handled by lowering before prepare_output")
                        }
                        MapTarget::Gpr(gpr) => ValueLoc::Mapped(gpr),
                        MapTarget::XmmShared { .. } | MapTarget::XmmExclusive(..) => {
                            let temp = temp_allocator.allocate().unwrap_or_else(|_| {
                                panic!("prepare_output could not allocate temp GPR")
                            });
                            ValueLoc::Temp(Rc::new(temp))
                        }
                    };
                    entry.insert(src.clone());
                    PreparedOutput::new(src, *target)
                }
            });
        }

        // TODO: return the instruction context
        InstructionContext {
            inputs: prepared_inputs.try_into().unwrap_or_else(|_| {
                unreachable!(
                    "we push one prepared_input for each input, hence should be the same size"
                )
            }),
            output: prepared_output.expect("output must be present"),
            clobber_restore,
        }
    }
}

struct InstructionContext<'a, const NI: usize, const NCT: usize> {
    inputs: [PreparedInput<'a>; NI],
    output: PreparedOutput<'a>,
    clobber_restore: Vec<PreparedOutput<'a>>,
}
