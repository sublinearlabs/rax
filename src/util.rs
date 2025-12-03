/// Returns a u64 number with n 1's
/// for example:
///     n = 2 -> 00..011
///     n = 4 -> 00..1111
pub(crate) fn mask(n: u8) -> u64 {
    if n == 64 {
        return u64::MAX;
    }

    (1 << n) - 1
}
