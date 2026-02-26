use crate::ir::{AtomicWidth, BlockId, IrBuilder, IrType, MemWidth, ValueId};

impl IrBuilder {
    // Assert a value has the expected IR type.
    pub(crate) fn expect_type(&self, v: ValueId, ty: IrType) {
        let actual = self.value_type(v);
        if actual != ty {
            panic!("type mismatch: expected {:?}, got {:?}", ty, actual);
        }
    }

    // Assert two values have the same IR type.
    pub(crate) fn expect_same_type(&self, a: ValueId, b: ValueId) {
        let a_ty = self.value_type(a);
        let b_ty = self.value_type(b);
        if a_ty != b_ty {
            panic!("type mismatch: expected {:?}, got {:?}", a_ty, b_ty);
        }
    }

    // Assert block argument count and types match the target block signature.
    pub(crate) fn check_block_args(&self, block: BlockId, args: &[ValueId]) {
        let block = &self.func.blocks[block.0 as usize];
        if block.args.len() != args.len() {
            panic!("block arg count mismatch");
        }
        for (arg, param) in args.iter().zip(block.args.iter()) {
            let expected = self.value_type(*param);
            let actual = self.value_type(*arg);
            if expected != actual {
                panic!(
                    "block arg type mismatch: expected {:?}, got {:?}",
                    expected, actual
                );
            }
        }
    }

    // Assert the requested memory width maps to the provided IR type.
    pub(crate) fn expect_mem_width_type(&self, width: MemWidth, ty: IrType) {
        let expected = match width {
            MemWidth::W8 => IrType::I8,
            MemWidth::W16 => IrType::I16,
            MemWidth::W32 => IrType::I32,
            MemWidth::W64 => IrType::I64,
        };
        if expected != ty {
            panic!(
                "mem width type mismatch: expected {:?}, got {:?}",
                expected, ty
            );
        }
    }

    // Assert the value type matches the requested memory width.
    pub(crate) fn expect_mem_width_value(&self, width: MemWidth, v: ValueId) {
        let expected = match width {
            MemWidth::W8 => IrType::I8,
            MemWidth::W16 => IrType::I16,
            MemWidth::W32 => IrType::I32,
            MemWidth::W64 => IrType::I64,
        };
        self.expect_type(v, expected);
    }

    // Assert the atomic width maps to the provided IR type.
    pub(crate) fn expect_atomic_width_type(&self, width: AtomicWidth, ty: IrType) {
        let expected = match width {
            AtomicWidth::W => IrType::I32,
            AtomicWidth::D => IrType::I64,
        };
        if expected != ty {
            panic!(
                "atomic width type mismatch: expected {:?}, got {:?}",
                expected, ty
            );
        }
    }

    // Assert the value type matches the atomic width.
    pub(crate) fn expect_atomic_width_value(&self, width: AtomicWidth, v: ValueId) {
        let expected = match width {
            AtomicWidth::W => IrType::I32,
            AtomicWidth::D => IrType::I64,
        };
        self.expect_type(v, expected);
    }
}
