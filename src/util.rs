use std::convert::TryInto;

/// Represents a Universal Disk Image Format (UDIF) checksum
/// structure.
#[derive(Debug)]
pub struct UDIFChecksum {
    /// Data fork
    pub fork_type: u32,
    /// Checksum information
    pub size: u32,
    /// Up to 128-bytes ( 32 * 4 ) of checksum
    pub data: Vec<u8>,
}

impl UDIFChecksum {
    pub fn to_be_bytes(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut fork_type = self.fork_type.to_be_bytes().to_vec();
        buffer.append(&mut fork_type);
        let mut size = self.size.to_be_bytes().to_vec();
        buffer.append(&mut size);
        let mut data = self.data;
        buffer.append(&mut data);

        buffer
    }
}

/// Create a u32 from big-endian ordered bytes
pub fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

/// Create a u64 from big-endian ordered bytes
pub fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

/// Create a u128 from big-endian ordered bytes
pub fn read_be_u128(input: &mut &[u8]) -> u128 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u128>());
    *input = rest;
    u128::from_be_bytes(int_bytes.try_into().unwrap())
}
