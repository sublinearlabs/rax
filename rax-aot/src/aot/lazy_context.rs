use std::{
    cell::RefCell,
    collections::{btree_map::Values, hash_map::Entry, HashMap},
    rc::Rc,
};

use crate::{
    aot::{
        instruction_context::{PreparedOutput, ValueLoc},
        register_mapping::{MapTarget, XmmLane},
        temp_alloc::TempAllocator,
        translator::Translator,
    },
    emit_asm,
};

// TODO: properly comment this code base, make it extremely readable
// TODO: have concrete type for Id, that type check the possible values

enum ContextParamType {
    Input,
    Output,
    Clobber,
}

struct ContextParam {
    target: MapTarget,
    param_type: ContextParamType,
    id: u8,
}

struct InstructionContext<'a> {
    cache: HashMap<MapTarget, ValueLoc<'a>>,
    translator: &'a Translator,
    prepared_outputs: Vec<PreparedOutput<'a>>,
}

impl<'a> InstructionContext<'a> {
    // TODO: document that this panics when the target is const zero
    fn resolve_target(&mut self, target: &MapTarget, target_type: ContextParamType) -> u8 {
        // this should try to get it from the cache
        // if it is unable to do so, it should resolve it manually
        // and then it should update the cache entry
        // does it need to know what type or parameter it is?
        //
        // how does one resolve the target for a clobber?
        // what does that even mean?
        // how do we know if we have moved for a clobber already?
        // it is assumed that resolve target will be coming from .id()
        // a clobber is usually an x86 gpr, so if it has an entry then
        // we know we have handled the clobber, but if it is not
        // then we know that we haven't performed the clobber
        //
        // the state of direct mapped targets, those are already gprs
        // as such there should be no entry for them usually right?
        // the xmms will get mapped to temp gprs not direct mapped gprs
        // hence the only reason why there should be some entry is when
        // the value has been remapped
        // this is how we handle clobber type
        //
        // for input, we check if there is an entry, if there is we return it
        // if there isn't, depending on the type, we might emit some movement instructions
        // then we update the cache in those cases
        //
        // for output, we check if there is an entry also
        // depending on the type we decide how we want to emit instructions
        //
        // given all of these, what is the strategy?
        // I think I have to handle each type

        match target_type {
            ContextParamType::Input => self.resolve_input_target(*target),
            ContextParamType::Output => self.resolve_output_target(*target),
            ContextParamType::Clobber => self.resolve_clobber_target(*target),
        }
    }

    fn resolve_input_target(&mut self, target: MapTarget) -> u8 {
        // I get the value of the input or I create a new input temp
        let value_loc = match self.cache.entry(target) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => match target {
                MapTarget::ConstZero => ValueLoc::ConstZero,
                MapTarget::Gpr(x86_gpr) => ValueLoc::Mapped(x86_gpr),
                MapTarget::XmmExclusive(reg)
                | MapTarget::XmmShared {
                    reg,
                    lane: XmmLane::Low,
                } => {
                    let val = Self::alloc_temp(self.translator);
                    emit_asm!(self.translator ; movq Rq(val.id()), Rx(reg.id()));
                    entry.insert(val.clone());
                    val
                }
                MapTarget::XmmShared {
                    reg,
                    lane: XmmLane::High,
                } => {
                    let val = Self::alloc_temp(self.translator);
                    emit_asm!(self.translator ; pextrq Rq(val.id()), Rx(reg.id()), 1);
                    entry.insert(val.clone());
                    val
                }
            },
        };

        // but we usually do not want to call .id() on a zero input
        // so this cannot work
        // actually it can work, the assumption is that you should have
        // checked first before calling .id() on this
        // so we add documentation that this will panic
        value_loc.id()
    }

    fn resolve_output_target(&mut self, target: MapTarget) -> u8 {
        // tries to get from the cache
        // if no entry then we create a new one
        // we need a way to be able to write back tho
        // a number is not going to be enough
        // we can say the placeholder, but then it will need a way to
        // point back to the entry it is coming from
        // so that we can forward the argument

        let prepared_output = match self.cache.entry(target) {
            Entry::Occupied(entry) => PreparedOutput::new(entry.get().clone(), target),
            Entry::Vacant(entry) => {
                let src = match target {
                    MapTarget::ConstZero => ValueLoc::ConstZero,
                    MapTarget::Gpr(gpr) => ValueLoc::Mapped(gpr),
                    MapTarget::XmmShared { .. } | MapTarget::XmmExclusive(..) => {
                        Self::alloc_temp(self.translator)
                    }
                };
                entry.insert(src.clone());
                PreparedOutput::new(src, target)
            }
        };

        let id = prepared_output.id();
        self.prepared_outputs.push(prepared_output);
        id
    }

    fn resolve_clobber_target(&mut self, target: MapTarget) -> u8 {
        todo!()
    }

    fn alloc_temp(translator: &Translator) -> ValueLoc {
        let temp = translator
            .temp_pool
            .allocate()
            .unwrap_or_else(|_| panic!("instruction context could not allocate temp GPR"));
        ValueLoc::Temp(Rc::new(temp))
    }
}

// TODO: properly handle drop for instruction context
