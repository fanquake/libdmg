use super::util;
use super::xml::XMLError;
use super::blkx::{BlkxChunkEntry, BLKX_CHUNK_ENTRY_SIZE};

use base64::{decode};

const MISH_MAGIC: &str = "0x6D697368";
// used by libdmg-hfsplus, hdiutil just sets this to the partition number?
//const ENTIRE_DEVICE_DESCRIPTOR: u32 = 4_294_967_294;

/// Decoded from a base64 string
/// All fields are in big endian ordering to maintain compatiblity
/// with older versions of macOS.
#[derive(Debug)]
pub struct MishBlock {
    /// Magic - "mish" in ASCII
    pub signature: u32,
    /// Current version is 1
    pub version: u32,
    /// Starting disk sector in this blkx descriptor
    pub sector_number: u64,
    /// Number of disk sectors in this blkx descriptor
    pub sector_count: u64,

    /// Start of raw data
    pub data_offset: u64,
    /// Size of the buffer in sectors needed to decompress
    pub buffers_needed: u32,
    /// Blocks descriptor
    pub block_descriptors: u32,

    /// Zeroed data
    pub reserved_1: u32,
    pub reserved_2: u32,
    pub reserved_3: u32,
    pub reserved_4: u32,
    pub reserved_5: u32,
    pub reserved_6: u32,

    /// UDIF Checksum - see util:UDIFChecksum
    pub checksum: util::UDIFChecksum,

    /// Number of entries in the blkx run table afterwards
    pub number_block_chunks: u32,
    /// [ num_block_chunks * blkxChunkEntry (40 bytes each)]
    pub block_entries: Vec<BlkxChunkEntry>,
}

impl MishBlock {
    pub fn from_base64(encoded: String) -> Result<MishBlock, XMLError> {
        // trim leading and trailing whitespace, tabs and newlines
        let stripped = encoded.trim().replace("\t", "").replace("\n", "");

        let decoded = decode(&stripped)?;

        //println!("Decoded mish bytes: {:#?}", decoded[0..30].to_vec());
        MishBlock::from_be_bytes(decoded)
    }

    pub fn from_be_bytes(buffer: Vec<u8>) -> Result<MishBlock, XMLError> {
        let signature = util::read_be_u32(&mut &buffer[0..4]);

        if format!("{:#X}", signature) != MISH_MAGIC {
            return Err(XMLError::Mish("Invalid mish magic bytes".to_string()));
        }

        let build = MishBlock::build_block_entries(&buffer[204..]);

        let block_entries = match build {
            Ok(entries) => entries,
            Err(_e) => return Err(XMLError::Mish("Could not build block entries".to_string())),
        };

        Ok(MishBlock {
            signature,
            version: util::read_be_u32(&mut &buffer[4..8]),
            sector_number: util::read_be_u64(&mut &buffer[8..16]),
            sector_count: util::read_be_u64(&mut &buffer[16..24]),

            data_offset: util::read_be_u64(&mut &buffer[24..32]),
            buffers_needed: util::read_be_u32(&mut &buffer[32..36]),
            block_descriptors: util::read_be_u32(&mut &buffer[36..40]),

            reserved_1: util::read_be_u32(&mut &buffer[40..44]),
            reserved_2: util::read_be_u32(&mut &buffer[44..48]),
            reserved_3: util::read_be_u32(&mut &buffer[48..52]),
            reserved_4: util::read_be_u32(&mut &buffer[52..56]),
            reserved_5: util::read_be_u32(&mut &buffer[56..60]),
            reserved_6: util::read_be_u32(&mut &buffer[60..64]),

            checksum: util::UDIFChecksum {
                fork_type: util::read_be_u32(&mut &buffer[64..68]),
                size: util::read_be_u32(&mut &buffer[68..72]),
                data: buffer[72..200].to_vec(),
            },

            number_block_chunks: util::read_be_u32(&mut &buffer[200..204]),
            block_entries,
        })
    }

    pub fn to_be_bytes(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut signature = self.signature.to_be_bytes().to_vec();
        buffer.append(&mut signature);
        let mut version = self.version.to_be_bytes().to_vec();
        buffer.append(&mut version);
        let mut sector_number = self.sector_number.to_be_bytes().to_vec();
        buffer.append(&mut sector_number);
        let mut sector_count = self.sector_count.to_be_bytes().to_vec();
        buffer.append(&mut sector_count);
        let mut data_offset = self.data_offset.to_be_bytes().to_vec();
        buffer.append(&mut data_offset);
        let mut buffers_needed = self.buffers_needed.to_be_bytes().to_vec();
        buffer.append(&mut buffers_needed);
        let mut block_descriptors = self.block_descriptors.to_be_bytes().to_vec();
        buffer.append(&mut block_descriptors);
        let mut reserved_1 = self.reserved_1.to_be_bytes().to_vec();
        buffer.append(&mut reserved_1);
        let mut reserved_2 = self.reserved_2.to_be_bytes().to_vec();
        buffer.append(&mut reserved_2);
        let mut reserved_3 = self.reserved_3.to_be_bytes().to_vec();
        buffer.append(&mut reserved_3);
        let mut reserved_4 = self.reserved_4.to_be_bytes().to_vec();
        buffer.append(&mut reserved_4);
        let mut reserved_5 = self.reserved_5.to_be_bytes().to_vec();
        buffer.append(&mut reserved_5);
        let mut reserved_6 = self.reserved_6.to_be_bytes().to_vec();
        buffer.append(&mut reserved_6);

        let mut checksum = self.checksum.to_be_bytes();
        buffer.append(&mut checksum);

        let mut number_block_chunks = self.number_block_chunks.to_be_bytes().to_vec();
        buffer.append(&mut number_block_chunks);
        //println!("buffer: {:#?}", buffer.len());
        //assert!(buffer.len() == 204);
        let mut block_entries: Vec<u8> = self.block_entries.into_iter().flat_map(|block| block.to_be_bytes()).collect();
        buffer.append(&mut block_entries);

        buffer
    }

    fn build_block_entries(buffer: &[u8]) -> Result<Vec<BlkxChunkEntry>, XMLError> {
        buffer
            .chunks_exact(BLKX_CHUNK_ENTRY_SIZE)
            .map(|c| BlkxChunkEntry::new(c))
            .collect()
    }
}