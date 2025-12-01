/// Returns a u32 number with n 1's
/// for example:
///     n = 2 -> 00..011
///     n = 4 -> 00..1111
pub(crate) fn mask(n: u8) -> u32 {
    if n == 32 {
        return u32::MAX;
    }

    (1 << n) - 1
}
