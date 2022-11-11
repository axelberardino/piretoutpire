// Divide two integer, but round it to the ceil instead of the floor.
// Dividing by 0 will panic.
pub const fn div_ceil(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

// Compute the distance between 2 crc/id.
// It's done by using a xor.
pub const fn distance(a: u32, b: u32) -> u32 {
    a ^ b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_testing() {
        assert_eq!(0, div_ceil(0, 1));
        assert_eq!(1, div_ceil(1, 1));
        assert_eq!(1, div_ceil(2, 2));

        assert_eq!(2, div_ceil(3, 2));
        assert_eq!(2, div_ceil(4, 2));
        assert_eq!(3, div_ceil(5, 2));
        assert_eq!(3, div_ceil(6, 2));
        assert_eq!(4, div_ceil(7, 2));
    }
}
