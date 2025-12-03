/// Mask with the lowest `n` bits set (0–32).
pub fn mask32(n: u8) -> u32 {
    if n == 32 {
        return u32::MAX;
    }

    (1 << n) - 1
}

/// Mask with the lowest `n` bits set (0–64).
pub fn mask(n: u8) -> u64 {
    if n == 64 {
        return u64::MAX;
    }

    (1 << n) - 1
}

/// Sign-extend the low `bit_count` bits of `val` into a u64.
pub fn sext(val: u32, bit_count: usize) -> u64 {
    debug_assert_eq!(val as u64 >> bit_count, 0, "upper bits must be zero");

    // bit count must be at least 1 and at most 32
    if bit_count == 0 || bit_count > 32 {
        panic!("invalid bit count");
    }

    let val = val as u64;

    // bit_count represents the length of the binary sequence we plan to extend
    // right_shift to erase all elements other than the sign bit
    let sign_bit = val >> (bit_count - 1);

    // pad higher bits withs 1s
    if sign_bit == 1 {
        return val | (u64::MAX << bit_count);
    }

    val
}

#[cfg(test)]
mod test {
    use crate::util::{mask, sext};

    #[test]
    fn test_mask_basic() {
        assert_eq!(mask(0), 0);
        assert_eq!(mask(1), 0b1);
        assert_eq!(mask(2), 0b11);
        assert_eq!(mask(3), 0b111);
    }

    #[test]
    fn test_mask_midrange() {
        assert_eq!(mask(8), 0xFF);
        assert_eq!(mask(10), 0x3FF);
        assert_eq!(mask(16), 0xFFFF);
    }

    #[test]
    fn test_mask_upper_bits() {
        assert_eq!(mask(63), 0x7FFF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn test_mask_full_width() {
        assert_eq!(mask(64), u64::MAX);
    }
    #[test]

    fn test_sext_positive_values() {
        // bit_count = 1 (sign bit is LSB)
        assert_eq!(sext(0b0, 1), 0);

        // bit_count = 3
        assert_eq!(sext(0b001, 3), 1);
        assert_eq!(sext(0b011, 3), 3);

        // bit_count = 6
        assert_eq!(sext(0b010101, 6), 0b010101);
    }

    #[test]
    fn test_sext_negative_values() {
        // bit_count = 3 (sign bit at position 2)
        // binary: 1_10 = -2 in 3-bit signed
        assert_eq!(sext(0b110, 3), -2_i64 as u64);

        // bit_count = 5 (sign bit at position 4)
        // binary: 1_0001 = -15 in 5-bit signed
        assert_eq!(sext(0b1_0001, 5), -15_i64 as u64);

        // bit_count = 8
        assert_eq!(sext(0xFF, 8), u64::MAX);

        // bit_count = 8: 0b1110_0000 = -32
        assert_eq!(sext(0b1110_0000, 8), -32_i64 as u64);
    }

    #[test]
    fn test_sext_boundary_cases() {
        // Smallest bit_count
        assert_eq!(sext(0b0, 1), 0);
        assert_eq!(sext(0b1, 1), u64::MAX); // 1-bit "-1"

        // Largest bit_count
        assert_eq!(sext(0x7FFF_FFFF, 32), 0x7FFF_FFFF); // positive max
        assert_eq!(sext(0x8000_0000, 32), 0xFFFF_FFFF_8000_0000); // negative
        assert_eq!(sext(0xFFFF_FFFF, 32), u64::MAX); // -1
    }
}
