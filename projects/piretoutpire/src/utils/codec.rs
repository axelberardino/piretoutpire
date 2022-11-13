use errors::{bail, AnyResult};
use std::net::SocketAddr;

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

// Convert a u32 list int u8 vec.
// Array must be short (< 256 values!).
// On error, it will panic.
//
// See `u32_list_to_u8_array` for a failable version.
pub fn u32_list_to_u8_array_unfailable(list: &[u32]) -> Vec<u8> {
    u32_list_to_u8_array(list).expect("msg")
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
        res.push(u8_array_to_u32(array.try_into()?));
    }

    Ok(res)
}

// Convert a String into a u8 array.
// Array is length(4) + str as bytes(n).
pub fn string_to_u8_array(str: String) -> Vec<u8> {
    let bytes_str = str.as_bytes();
    let mut res = Vec::with_capacity(str.len() + 4);
    res.extend(u32_to_u8_array(bytes_str.len() as u32));
    res.extend(bytes_str);
    res
}

// Convert a u8 array into a String.
// Array is length(4) + str as bytes(n).
pub fn u8_array_to_string(array: &[u8]) -> AnyResult<String> {
    if array.len() < 4 {
        bail!("string length prefix is invalid");
    }
    let slice: [u8; 4] = core::array::from_fn(|idx| array[idx]);
    let size = u8_array_to_u32(&slice);
    if (size as usize) != array.len() - 4 {
        bail!("invalid size expected {} but got {}", size, array.len())
    }
    let raw_str = array
        .iter()
        .skip(4)
        .take(size as usize)
        .map(|ch| *ch)
        .collect::<Vec<u8>>();
    Ok(String::from_utf8(raw_str)?)
}

// Convert a SocketAddr into a u8 array.
// Use the string representation of a socket addr.
pub fn addr_to_u8_array(addr: SocketAddr) -> Vec<u8> {
    string_to_u8_array(addr.to_string())
}

// Convert a u8 array into a SocketAddr.
// Addr is represented as a string, then converted into a socket addr.
pub fn u8_array_to_addr(array: &[u8]) -> AnyResult<SocketAddr> {
    let str_addr = u8_array_to_string(array)?;
    Ok(str_addr.parse()?)
}

#[cfg(test)]
#[path = "codec_test.rs"]
mod codec_test;
