// Convert a u32 into a 4*u8 vec.
pub fn u32_to_u8_array(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;

    [b1, b2, b3, b4]
}

// Convert a 4*u8 vec into a u32.
pub fn u8_array_to_u32(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_u32_to_u8() {
        assert_eq!([0, 0, 0, 0], u32_to_u8_array(0));
        assert_eq!([0, 0, 0, 1], u32_to_u8_array(1));
        assert_eq!([0, 0, 0, 255], u32_to_u8_array(255));

        assert_eq!([0, 0, 1, 0], u32_to_u8_array(256));
        assert_eq!([0, 0, 255, 255], u32_to_u8_array(65535));
        assert_eq!([0, 1, 0, 0], u32_to_u8_array(65536));

        assert_eq!([0, 255, 255, 255], u32_to_u8_array(16777215));
        assert_eq!([1, 0, 0, 0], u32_to_u8_array(16777216));

        assert_eq!([255, 255, 255, 255], u32_to_u8_array(4294967295));
    }

    #[test]
    fn test_convert_u8_to_u32() {
        assert_eq!(0, u8_array_to_u32(&[0, 0, 0, 0]));
        assert_eq!(1, u8_array_to_u32(&[0, 0, 0, 1]));
        assert_eq!(255, u8_array_to_u32(&[0, 0, 0, 255]));

        assert_eq!(256, u8_array_to_u32(&[0, 0, 1, 0]));
        assert_eq!(65535, u8_array_to_u32(&[0, 0, 255, 255]));
        assert_eq!(65536, u8_array_to_u32(&[0, 1, 0, 0]));

        assert_eq!(16777215, u8_array_to_u32(&[0, 255, 255, 255]));
        assert_eq!(16777216, u8_array_to_u32(&[1, 0, 0, 0]));

        assert_eq!(4294967295, u8_array_to_u32(&[255, 255, 255, 255]));
    }
}
