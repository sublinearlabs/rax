use dynasmrt::x64::Assembler;

use crate::aot::{register_mapping::RegisterMapping, temp_alloc::TempAllocator};

struct Translator {
    emitter: Assembler,
    reg_map: RegisterMapping,
    temp_allocator: TempAllocator,
}
