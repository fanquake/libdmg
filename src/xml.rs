use super::util;
use base64::decode;
use std::error;
use std::fmt;
use xmltree;

/// Ways that XML parsing might fail
#[derive(Debug)]
pub enum XMLError {
    Base64(String),
    Mish(String),
    Partition(String),
    XML(String),
}

impl fmt::Display for XMLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XMLError::Base64(e) => fmt::Display::fmt(e, f),
            XMLError::Mish(e) => fmt::Display::fmt(e, f),
            XMLError::Partition(e) => fmt::Display::fmt(e, f),
            XMLError::XML(e) => fmt::Display::fmt(e, f),
        }
    }
}

impl error::Error for XMLError {
    fn description(&self) -> &str {
        match self {
            XMLError::Base64(e) => e,
            XMLError::Mish(e) => e,
            XMLError::Partition(e) => e,
            XMLError::XML(e) => e,
        }
    }
}

impl From<base64::DecodeError> for XMLError {
    fn from(e: base64::DecodeError) -> XMLError {
        XMLError::Base64(e.to_string())
    }
}

impl From<xmltree::ParseError> for XMLError {
    fn from(e: xmltree::ParseError) -> XMLError {
        XMLError::XML(e.to_string())
    }
}

#[derive(Debug)]
pub struct PartitionEntry {
    attributes: String, // turn this into hex or some int?
    cf_name: String,
    data: MishBlock, // base64 encoded string, decode into mish block
    id: i32,         // this can be -1...n
    name: String,    // always seems to be the same as cf_name
}

impl PartitionEntry {
    pub fn new(element: &xmltree::Element) -> Result<PartitionEntry, XMLError> {
        let children = &element.children;

        // TODO: extract strings and turn static?
        let attributes = PartitionEntry::find_index_for(String::from("Attributes"), &children)?;
        let cf_name = PartitionEntry::find_index_for(String::from("CFName"), &children)?;
        let data = PartitionEntry::find_index_for(String::from("Data"), &children)?;
        // TODO: yuck?
        let id: i32 = PartitionEntry::find_index_for(String::from("ID"), &children)?
            .parse()
            .unwrap();
        let name = PartitionEntry::find_index_for(String::from("Name"), &children)?;

        Ok(PartitionEntry {
            attributes,
            cf_name,
            data: MishBlock::new_from_base_64(data)?,
            id,
            name,
        })
    }

    fn find_index_for(key: String, elements: &[xmltree::Element]) -> Result<String, XMLError> {
        let key_index = elements
            .iter()
            .position(|x| x.text == Some(key.clone()))
            .unwrap();

        // This assumes that the XML elements are always ordered correctly
        let value = &elements[key_index + 1];

        match &value.text {
            Some(text) => Ok(text.to_string()),
            None => Err(XMLError::Partition(
                "Could not retreive value for key".to_string(),
            )),
        }
    }
}

const BLKX_CHUNK_ENTRY_SIZE: usize = 40;

#[derive(Debug)]
pub struct BlkxChunkEntry {
    /// Compression type used or entry type
    entry_type: u32,
    /// "+beg" or "+end" if entry_type is comment (0x7FFFFFFE). Else reserved
    comment: u32,
    /// Start sector of this chunk
    sector_number: u64,
    /// Number of sectors in this chunk
    sector_count: u64,
    /// Start of chunk in data fork
    compressed_offset: u64,
    /// Count of bytes of chunk, in data fork
    compressed_length: u64,
}

impl BlkxChunkEntry {
    pub fn new(buffer: &[u8]) -> BlkxChunkEntry {
        BlkxChunkEntry {
            entry_type: util::read_be_u32(&mut &buffer[0..4]),
            comment: util::read_be_u32(&mut &buffer[4..8]),
            sector_number: util::read_be_u64(&mut &buffer[8..16]),
            sector_count: util::read_be_u64(&mut &buffer[16..24]),
            compressed_offset: util::read_be_u64(&mut &buffer[24..32]),
            compressed_length: util::read_be_u64(&mut &buffer[32..40]),
        }
    }
}

const MISH_MAGIC: &str = "0x6D697368";

/// Decoded from a base64 string
/// All fields are in big endian ordering to maintain compatiblity
/// with older versions of macOS.
#[derive(Debug)]
pub struct MishBlock {
    /// Magic - "mish" in ASCII
    signature: u32,
    /// Current version is 1
    version: u32,
    /// Starting disk sector in this blkx descriptor
    sector_number: u64,
    /// Number of disk sectors in this blkx descriptor
    sector_count: u64,

    /// Start of raw data
    data_offset: u64,
    /// Size of the buffer in sectors needed to decompress
    buffers_needed: u32,
    /// Number of block descriptors
    block_descriptors: u32,

    /// Zeroed data
    reserved_1: u32,
    reserved_2: u32,
    reserved_3: u32,
    reserved_4: u32,
    reserved_5: u32,
    reserved_6: u32,

    /// UDIF Checksum - see util:UDIFChecksum
    checksum: util::UDIFChecksum,

    /// Number of entries in the blkx run table afterwards
    number_block_chunks: u32,
    /// [ num_block_chunks * blkxChunkEntry (40 bytes each)]
    block_entries: Vec<BlkxChunkEntry>,
}

impl MishBlock {
    pub fn new_from_base_64(encoded: String) -> Result<MishBlock, XMLError> {
        // trim leading and trailing whitespace, tabs and newlines
        let stripped = encoded.trim().replace("\t", "").replace("\n", "");

        let decoded = decode(&stripped)?;

        MishBlock::new(decoded)
    }

    pub fn new(buffer: Vec<u8>) -> Result<MishBlock, XMLError> {
        let signature = util::read_be_u32(&mut &buffer[0..4]);
        if format!("{:#X}", signature) != MISH_MAGIC {
            return Err(XMLError::Mish("Invalid mish magic bytes".to_string()));
        }

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
            block_entries: MishBlock::build_block_entries(&buffer[204..]),
        })
    }

    fn build_block_entries(buffer: &[u8]) -> Vec<BlkxChunkEntry> {
        buffer
            .chunks_exact(BLKX_CHUNK_ENTRY_SIZE)
            .map(|c| BlkxChunkEntry::new(c))
            .collect()
    }
}

#[derive(Debug)]
pub struct PList {
    partitions: Vec<PartitionEntry>,
}

// Parse the XML plist data
pub fn parse_plist(data: Vec<u8>) -> Result<PList, XMLError> {
    let string = String::from_utf8(data).unwrap();

    let xml = xmltree::Element::parse(string.as_bytes())?;

    let outer_dict = xml.get_child("dict").unwrap();

    // check for the resource-fork key
    let resource_fork = outer_dict
        .get_child("key")
        .expect("Could not find resource-fork");
    let text = resource_fork
        .text
        .clone()
        .expect("Malformed resource-fork text");
    assert_eq!(text, "resource-fork");

    // get the array that contains the blk data entries
    let blk_array = outer_dict
        .get_child("dict")
        .unwrap()
        .get_child("array")
        .expect("Could not find blk data array");

    let partitions: Result<Vec<PartitionEntry>, XMLError> = blk_array
        .children
        .iter()
        .map(|child| PartitionEntry::new(child))
        .collect();

    match partitions {
        Ok(partitions) => Ok(PList { partitions }),
        Err(e) => Err(e),
    }
}
