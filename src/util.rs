fn mask32(n: u8) -> u32 {
    if n == 32 {
        return u32::MAX;
    }

    (1 << n) - 1
}

fn mask(n: u8) -> u64 {
    if n == 64 {
        return u64::MAX;
    }

    (1 << n) - 1
}
