/// Mask with the lowest `n` bits set (0–32).
fn mask32(n: u8) -> u32 {
    if n == 32 {
        return u32::MAX;
    }

    (1 << n) - 1
}

/// Mask with the lowest `n` bits set (0–64).
fn mask(n: u8) -> u64 {
    if n == 64 {
        return u64::MAX;
    }

    (1 << n) - 1
}

/// Sign-extend the low `bit_count` bits of `val` into a u64.
fn sext(val: u32, bit_count: usize) -> u64 {
    debug_assert_eq!(val >> bit_count, 0, "upper bits must be zero");

    // bit count must be at least 1 and at most 32
    if bit_count == 0 || bit_count > 32 {
        panic!("invalid bit count");
    }

    let val = val as u64;

    // bit_count represents the length of the binary sequence we plan to extend
    // right_shift to erase all elements other than the sign bit
    let sign_bit = (val >> (bit_count - 1)) & 1;

    // pad higher bits withs 1s
    if sign_bit == 1 {
        return val | (u64::MAX << bit_count);
    }

    val
}
