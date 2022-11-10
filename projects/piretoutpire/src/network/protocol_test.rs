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
