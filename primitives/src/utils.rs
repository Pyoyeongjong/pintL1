//! util functions or structs for blockchain

// const fn: Can executes in compile time!
pub const fn normalize_v(v: u64) -> Option<bool> {
    match v {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}
