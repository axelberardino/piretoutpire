use errors::{bail, AnyResult};

// Convert a u32 into a 4*u8 vec.
pub fn u32_to_u8_array(value: u32) -> [u8; 4] {
    let b1: u8 = ((value >> 24) & 0xff) as u8;
    let b2: u8 = ((value >> 16) & 0xff) as u8;
    let b3: u8 = ((value >> 8) & 0xff) as u8;
    let b4: u8 = (value & 0xff) as u8;

    [b1, b2, b3, b4]
}

// Convert a 4*u8 vec into a u32.
pub fn u8_array_to_u32(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}

// Convert a u32 list int u8 vec.
// Array must be short (< 256 values!).
pub fn u32_list_to_u8_array(list: &[u32]) -> AnyResult<Vec<u8>> {
    if list.len() > 256 {
        bail!("invalid size");
    }

    let mut buf = Vec::with_capacity(list.len() * 4 + 1);
    buf.push(list.len() as u8);
    for array in list.iter().map(|value| u32_to_u8_array(*value)) {
        buf.extend(array);
    }

    if buf.len() != list.len() * 4 + 1 {
        bail!(
            "error occured while decoding expected size {} but got {}",
            list.len() * 4 + 1,
            buf.len(),
        );
    }
    Ok(buf)
}

// Convert a u8 buffer into a u32 list.
// Array must be short (< 256 values!) and at least one byte (for the size) must
// be there.
pub fn u8_array_to_u32_list(array: &[u8]) -> AnyResult<Vec<u32>> {
    if array.len() < 1 {
        bail!("can't be empty");
    }
    let size = array[0] as usize;
    if size != (array.len() - 1) / 4 {
        bail!(
            "invalid size expected 4*{}={} but got {}",
            size,
            size * 4,
            array.len() - 1
        );
    }

    let mut res = Vec::with_capacity(size / 4);
    for array in array[1..].chunks(4) {
        dbg!(&array);
        res.push(u8_array_to_u32(array.try_into()?));
    }

    Ok(res)
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

    #[test]
    fn test_convert_u32_list_to_u8_array() -> AnyResult<()> {
        assert!(u32_list_to_u8_array(&[0; 257]).is_err());

        assert_eq!(vec![0], u32_list_to_u8_array(&[])?);
        assert_eq!(vec![1, 0, 0, 0, 9], u32_list_to_u8_array(&[9])?);
        assert_eq!(
            vec![4, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4],
            u32_list_to_u8_array(&[1, 2, 3, 4])?
        );

        Ok(())
    }

    #[test]
    fn test_convert_u8_array_to_u32_list() -> AnyResult<()> {
        assert!(u8_array_to_u32_list(&[]).is_err());
        assert!(u8_array_to_u32_list(&[200, 0, 0, 0, 1]).is_err());

        assert_eq!(vec![0; 0], u8_array_to_u32_list(&[0])?);
        assert_eq!(vec![9], u8_array_to_u32_list(&[1, 0, 0, 0, 9])?);
        assert_eq!(
            vec![1, 2, 3, 4],
            u8_array_to_u32_list(&[4, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])?
        );
        Ok(())
    }
}
