
// const fn: Can executes in compile time!
pub const fn normalize_v(v: u64) -> Option<bool> {
    let cmp = (v <= 1) as u64;
    Some(v % 2 == cmp)
}