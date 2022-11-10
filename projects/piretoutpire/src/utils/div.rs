// Divide two integer, but round it to the ceil instead of the floor.
// Dividing by 0 will panic.
pub const fn div_ceil(a: u64, b: u64) -> u64 {
    (a + b - 1) / b
}

// TODO add tests
