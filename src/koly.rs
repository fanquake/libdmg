use super::util;

const KOLY_MAGIC: &str = "0x6B6F6C79";

#[derive(Debug)]
pub struct KolyBlock {
    /// magic - 0x6B6F6C79 "koly" in ASCII
    pub magic: u32,
    /// currently 4
    pub version: u32,
    /// Should always be 512
    pub header_size: u32,
    pub flags: u32,

    /// where the running data fork starts (usually 0)
    pub running_data_fork_offset: u64,
    /// usually 0
    pub data_fork_offset: u64,
    /// size of data fork in bytes
    pub data_fork_length: u64,
    /// resource fork offset, if any
    pub source_fork_offset: u64,
    /// resource fork length, if any
    pub source_fork_length: u64,

    /// usually 1
    pub segment_number: u32,
    /// usually 1
    pub segment_count: u32,
    /// 128-bit number like a GUID, but not really
    pub segment_id: u128,

    /// 
    pub data_fork_checksum: util::UDIFChecksum,

    /// start of the .plist data
    pub xml_offset: u64,
    /// length of the .plist data
    pub xml_length: u64,

    /// 120 reserved bytes, zeroed
    pub reserved_one: Vec<u8>, //LargeArray,

    pub master_checksum: util::UDIFChecksum,

    /// commonly 1, we're using 2
    pub image_variant: u32,
    /// size of DMG when expanded, in sectors
    pub sector_count: u64,

    pub reserved_two: u32,
    pub reserved_three: u32,
    pub reserved_four: u32,
}

impl KolyBlock {
    // given a 512 byte long buffer of [u8] create a UDIF resource
    pub fn new(buffer: Vec<u8>) -> Result<KolyBlock, &'static str> {

        // sanity check that we've got 512 bytes
        // and that the first 4 are the koly magic
        assert_eq!(buffer.len(), 512);
        let magic = util::read_be_u32(&mut &buffer[0..4]);
        assert_eq!(format!("{:#X}", magic), KOLY_MAGIC);

        Ok(KolyBlock {
            magic: util::read_be_u32(&mut &buffer[0..4]),
            version: util::read_be_u32(&mut &buffer[4..8]),
            header_size: util::read_be_u32(&mut &buffer[8..12]),
            flags: util::read_be_u32(&mut &buffer[12..16]),

            running_data_fork_offset: util::read_be_u64(&mut &buffer[16..24]),
            data_fork_offset: util::read_be_u64(&mut &buffer[24..32]),
            data_fork_length: util::read_be_u64(&mut &buffer[32..40]),
            source_fork_offset: util::read_be_u64(&mut &buffer[40..48]),
            source_fork_length: util::read_be_u64(&mut &buffer[48..56]),

            segment_number: util::read_be_u32(&mut &buffer[56..60]),
            segment_count: util::read_be_u32(&mut &buffer[60..64]),

            segment_id: util::read_be_u128(&mut &buffer[64..80]),

            data_fork_checksum: util::UDIFChecksum {
                fork_type: util::read_be_u32(&mut &buffer[80..84]),
                size: util::read_be_u32(&mut &buffer[84..88]),
                data: util::from_buffer(&buffer[88..216]),
            },

            xml_offset: util::read_be_u64(&mut &buffer[216..224]),
            xml_length: util::read_be_u64(&mut &buffer[224..232]),

            reserved_one: vec![0u8; 120],

            master_checksum: util::UDIFChecksum {
                fork_type: util::read_be_u32(&mut &buffer[352..356]),
                size: util::read_be_u32(&mut &buffer[356..360]),
                data: util::from_buffer(&buffer[360..488]),
            },

            image_variant: util::read_be_u32(&mut &buffer[488..492]),
            sector_count: util::read_be_u64(&mut &buffer[492..500]),

            reserved_two: util::read_be_u32(&mut &buffer[500..504]),
            reserved_three: util::read_be_u32(&mut &buffer[504..508]),
            reserved_four: util::read_be_u32(&mut &buffer[508..512]),
        })
    }
}
