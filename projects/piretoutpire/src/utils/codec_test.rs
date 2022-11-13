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

#[test]
fn test_convert_string_to_u8_array() -> AnyResult<()> {
    assert_eq!(vec![0, 0, 0, 0], string_to_u8_array("".to_owned()));
    assert_eq!(vec![0, 0, 0, 1, 97], string_to_u8_array("a".to_owned()));
    assert_eq!(
        vec![0, 0, 0, 11, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100],
        string_to_u8_array("Hello world".to_owned())
    );
    Ok(())
}

#[test]
fn test_convert_u8_array_to_string() -> AnyResult<()> {
    assert!(u8_array_to_string(&[]).is_err());
    assert!(u8_array_to_string(&[0]).is_err());
    assert!(u8_array_to_string(&[0, 0]).is_err());
    assert!(u8_array_to_string(&[0, 0, 0]).is_err());
    assert!(u8_array_to_string(&[0, 0, 0, 35]).is_err());

    assert_eq!("", u8_array_to_string(&[0, 0, 0, 0])?);
    assert_eq!("a", u8_array_to_string(&[0, 0, 0, 1, 97])?);
    assert_eq!(
        "Hello world",
        u8_array_to_string(&[0, 0, 0, 11, 72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100])?
    );
    Ok(())
}

#[test]
fn test_convert_addr_to_u8_array() -> AnyResult<()> {
    assert_eq!(
        vec![0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48],
        string_to_u8_array("127.0.0.1:4000".parse()?)
    );
    Ok(())
}

#[test]
fn test_convert_u8_array_to_addr() -> AnyResult<()> {
    assert!(u8_array_to_addr(&[0, 0, 08]).is_err());
    assert!(u8_array_to_addr(&[0, 0, 0, 14, 49, 50, 52, 48, 48, 48]).is_err());
    assert!(u8_array_to_addr(&[0, 0, 0, 3, 97, 97, 97]).is_err()); // "aaa".parse()

    assert_eq!(
        "127.0.0.1:4000".parse::<SocketAddr>()?,
        u8_array_to_addr(&[0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48])?
    );

    Ok(())
}
