use dynasmrt::x64::Assembler;

use crate::aot::{register_mapping::RegisterMapping, temp_alloc::TempAllocator};

struct Translator {
    emitter: Assembler,
    reg_map: RegisterMapping,
    temp_allocator: TempAllocator,
}

impl Translator {}

// what does it mean to prepare an input and an output?
// let us focus on the input
// for an input, it either exists in gpr or not
// if it doesn't then we need to send back a gpr reg (which we get from temp)
// seems I'd need a combined type that holds either a temp guard or a gpr directly
//
// for output I'd like a new type that implements a write back function
// if one doesn't write back before drop then it panics
//
// hence it seems I need two new types
// something Input and Output,
// Input will be an enum i.e. FromTemp and FromGpr
// Output will be a struct with that holds src and destination I believe
// with a flag to signify if it has been written?
// then writeback updates that flag
