use std::convert::TryInto;

#[derive(Debug)]
pub struct UDIFChecksum {
    /// data fork
    pub fork_type: u32,
    /// checksum information
    pub size: u32,
    /// up to 128-bytes ( 32 * 4 ) of checksum
    pub data: Vec<u8>,
}

pub fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn read_be_u128(input: &mut &[u8]) -> u128 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u128>());
    *input = rest;
    u128::from_be_bytes(int_bytes.try_into().unwrap())
}
