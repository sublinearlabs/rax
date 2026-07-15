use std::{
    cell::RefCell,
    collections::{HashMap, btree_map::Values, hash_map::Entry}, rc::Rc,
};

use crate::{aot::{instruction_context::ValueLoc, register_mapping::{MapTarget, XmmLane}, temp_alloc::TempAllocator, translator::Translator}, emit_asm};

enum ContextParamType {
    Input,
    Output,
    Clobber,
}

struct ContextParam {
    target: MapTarget,
    param_type: ContextParamType,
}

struct InstructionContext<'a> {
    cache: HashMap<MapTarget, ValueLoc<'a>>,
    translator: &'a Translator
}

impl<'a> InstructionContext<'a> {
    fn resolve_target(&mut self, target: &MapTarget, target_type: ContextParamType) {
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

    fn resolve_input_target(&mut self, target: MapTarget, translator: &mut Translator, temp_allocator: &'a TempAllocator) {
        // we have a register that is supposed to behave like an input
        // we need to know what location we are supposed to use for it
        let value_loc = match self.cache.entry(target) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => match target {
                MapTarget::ConstZero => ValueLoc::ConstZero,
                MapTarget::Gpr(x86_gpr) => ValueLoc::Mapped(x86_gpr),
                MapTarget::XmmExclusive(reg) | MapTarget::XmmShared { reg, lane: XmmLane::Low } => {
                    let val = Self::alloc_temp(temp_allocator);
                    emit_asm!(self.translator ; movq Rq(val.id()), Rx(reg.id()));
                    entry.insert(val.clone());
                    val
                }
                MapTarget::XmmShared { reg, lane: XmmLane::High } => {
                    let val = Self::alloc_temp(temp_allocator);
                    dynasm!(translator.emitter ; pextrq Rq(val.id()), Rx(reg.id()), 1);
                    entry.insert(val.clone());
                    val
                }
            }
        }
        todo!()
    }

    fn resolve_output_target(&mut self, target: MapTarget) {
        todo!()
    }

    fn resolve_clobber_target(&mut self, target: MapTarget) {
        todo!()
    }

    fn alloc_temp(temp_allocator: &'a TempAllocator) -> ValueLoc {
        let temp = temp_allocator.allocate().unwrap_or_else(|_| panic!("instruction context could not allocate temp GPR"));
        ValueLoc::Temp(Rc::new(temp))
    }
}

// TODO: properly handle drop for instruction context
