// Divide two integer, but round it to the ceil instead of the floor.
// Dividing by 0 will panic.
pub const fn div_ceil(lhs: u32, rhs: u32) -> u32 {
    (lhs + rhs - 1) / rhs
}

// Take the middle of two numbers, without overflowing.
// Note that lhs must be less than rhs.
pub const fn middle_point(lhs: u32, rhs: u32) -> u32 {
    lhs + (rhs - lhs) / 2
}

// Compute the distance between 2 crc/id.
// It's done by using a xor.
pub const fn distance(lhs: u32, rhs: u32) -> u32 {
    lhs ^ rhs
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

    #[test]
    fn test_middle_point() {
        assert_eq!(0, middle_point(0, 0));
        assert_eq!(0, middle_point(0, 1));
        assert_eq!(1, middle_point(0, 2));

        assert_eq!(180, middle_point(120, 240));
        assert_eq!(180, middle_point(120, 241));
        assert_eq!(300, middle_point(300, 300));

        assert_eq!(u32::MAX - 1, middle_point(u32::MAX - 2, u32::MAX));
    }
}
