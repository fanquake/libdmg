use super::util;
use super::xml::XMLError;

pub const BLKX_CHUNK_ENTRY_SIZE: usize = 40;

/// DMG blxx types
#[derive(Debug)]
pub enum DmgBlxx {
    /// Zero fill - 0x00000000
    ZeroFill,
    /// RAW or UNLL compression (uncompressed) - 0x00000001
    RawOrNullCompression,
    /// Ignored/unknown - 0x00000002
    IgnoredOrUnknown,
    /// Apple data compression - 0x80000004
    AppleCompression,
    /// zLib data compression - 0x80000005
    ZLibCompression,
    /// bz2lib data compression - 0x80000006
    Bz2Compression,
    /// No blocks - Comment: +beg and +end - 0x7FFFFFFE
    Comment,
    /// No blocks - Identifies the last blxx entry - 0xFFFFFFFF
    LastEntry,
}

impl DmgBlxx {
    /// Convert big endian bytes into a DMG blxx type
    pub fn from_u32(be_bytes: &mut &[u8]) -> Option<DmgBlxx> {
        let uint = util::read_be_u32(be_bytes);

        match uint {
            0 => Some(DmgBlxx::ZeroFill),
            1 => Some(DmgBlxx::RawOrNullCompression),
            2 => Some(DmgBlxx::IgnoredOrUnknown),
            2_147_483_652 => Some(DmgBlxx::AppleCompression),
            2_147_483_653 => Some(DmgBlxx::ZLibCompression),
            2_147_483_654 => Some(DmgBlxx::Bz2Compression),
            4_294_967_294 => Some(DmgBlxx::Comment),
            4_294_967_295 => Some(DmgBlxx::LastEntry),
            _ => None,
        }
    }

    pub fn to_be_bytes(self) -> Vec<u8> {
        let val = match self {
            DmgBlxx::ZeroFill => 0u32.to_be_bytes(),
            DmgBlxx::RawOrNullCompression => 1u32.to_be_bytes(),
            DmgBlxx::IgnoredOrUnknown => 2u32.to_be_bytes(),
            DmgBlxx::AppleCompression => 2_147_483_652u32.to_be_bytes(),
            DmgBlxx::ZLibCompression => 2_147_483_653u32.to_be_bytes(),
            DmgBlxx::Bz2Compression => 2_147_483_654u32.to_be_bytes(),
            DmgBlxx::Comment => 4_294_967_294u32.to_be_bytes(),
            DmgBlxx::LastEntry => 4_294_967_295u32.to_be_bytes(),
        };
        val.to_vec()
    }
}

#[derive(Debug)]
pub struct BlkxChunkEntry {
    /// Compression type used or entry type
    pub entry_type: DmgBlxx,
    /// "+beg" or "+end" if entry_type is comment (0x7FFFFFFE). Else reserved
    pub comment: u32,
    /// Start sector of this chunk
    pub sector_number: u64,
    /// Number of sectors in this chunk
    pub sector_count: u64,
    /// Start of chunk in data fork
    pub compressed_offset: u64,
    /// Count of bytes of chunk, in data fork
    pub compressed_length: u64,
}

impl BlkxChunkEntry {
    pub fn new(buffer: &[u8]) -> Result<BlkxChunkEntry, XMLError> {
        let entry = DmgBlxx::from_u32(&mut &buffer[0..4]);
        //println!("u32: {:#?}", util::read_be_u32(&mut &buffer[0..4]));
        //println!("entry: {:#?}", entry);
        //println!("buffer: {:#?}", buffer.clone());

        let entry_type = match entry {
            Some(entry) => entry,
            None => return Err(XMLError::Blxx("Could not identify blxx type".to_string())),
        };

        Ok(BlkxChunkEntry {
            entry_type,
            comment: util::read_be_u32(&mut &buffer[4..8]),
            sector_number: util::read_be_u64(&mut &buffer[8..16]),
            sector_count: util::read_be_u64(&mut &buffer[16..24]),
            compressed_offset: util::read_be_u64(&mut &buffer[24..32]),
            compressed_length: util::read_be_u64(&mut &buffer[32..40]),
        })
    }

    pub fn to_be_bytes(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut entry_type = self.entry_type.to_be_bytes().to_vec();
        //println!("entry type: {:#?}", entry_type);
        buffer.append(&mut entry_type);
        let mut comment = self.comment.to_be_bytes().to_vec();
        buffer.append(&mut comment);
        let mut sector_number = self.sector_number.to_be_bytes().to_vec();
        buffer.append(&mut sector_number);
        let mut sector_count = self.sector_count.to_be_bytes().to_vec();
        buffer.append(&mut sector_count);
        let mut compressed_offset = self.compressed_offset.to_be_bytes().to_vec();
        buffer.append(&mut compressed_offset);
        let mut compressed_length = self.compressed_length.to_be_bytes().to_vec();
        buffer.append(&mut compressed_length);

        buffer
    }
}