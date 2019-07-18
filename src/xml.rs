use base64::{decode};
use super::util;
use xmltree;

#[derive(Debug)]
pub struct PartitionEntry {
    attributes: String, // turn this into hex or some int?
    cf_name: String,
    data: MishBlock, // base64 encoded string, decode into mish block
    id: i32, // this can be -1...n
    name: String, // always seems to be the same as cf_name
}

impl PartitionEntry {

    pub fn new(element: xmltree::Element) -> Result<PartitionEntry, &'static str> {

        let children = &element.children;

        // TODO: extract strings and turn static?
        let attributes = PartitionEntry::find_index_for(String::from("Attributes"), &children)?;
        let cf_name = PartitionEntry::find_index_for(String::from("CFName"), &children)?;
        let data = PartitionEntry::find_index_for(String::from("Data"), &children)?;
        // TODO: yuck?
        let id: i32 = PartitionEntry::find_index_for(String::from("ID"), &children)?.parse().unwrap();
        let name = PartitionEntry::find_index_for(String::from("Name"), &children)?;

        Ok(PartitionEntry {
            attributes,
            cf_name,
            data: MishBlock::new_from_base_64(data)?,
            id,
            name,
        })
    }

    fn find_index_for(key: String, elements: &[xmltree::Element]) -> Result<String, &'static str> {

        let key_index = elements.iter().position(|x| x.text.clone() == Some(key.clone())).unwrap();
        
        // This assumes that the XML elements are always ordered correctly
        let value = &elements[key_index + 1];

        match &value.text {
            Some(text) => Ok(text.to_string()),
            None => Err("Could not retreive value for key"),
        }
    }
}

const BLKX_CHUNK_ENTRY_SIZE: usize = 40;

#[derive(Debug)]
pub struct BlkxChunkEntry {
    entry_type: u32, // compression type used
    comment: u32,
    sector_number: u64,
    sector_count: u64,
    compressed_offset: u64,
    compressed_length: u64,
}

impl BlkxChunkEntry {
    pub fn new(buffer: Vec<u8>) -> Result<BlkxChunkEntry, &'static str> {
        Ok(BlkxChunkEntry {
            entry_type: util::read_be_u32(&mut &buffer[0..4]),
            comment: util::read_be_u32(&mut &buffer[4..8]),
            sector_number: util::read_be_u64(&mut &buffer[8..16]),
            sector_count: util::read_be_u64(&mut &buffer[16..24]),
            compressed_offset: util::read_be_u64(&mut &buffer[24..32]),
            compressed_length: util::read_be_u64(&mut &buffer[32..40]),
        })
    }
}

const MISH_MAGIC: &str = "0x6D697368";

#[derive(Debug)]
pub struct MishBlock {
    signature: u32,
    version: u32,
    sector_number: u64,
    sector_count: u64,

    data_offset: u64,
    buffers_needed: u32,
    block_descriptors: u32,

    reserved_1: u32,
    reserved_2: u32,
    reserved_3: u32,
    reserved_4: u32,
    reserved_5: u32,
    reserved_6: u32,

    checksum: util::UDIFChecksum,

    number_block_chunks: u32,
    block_entries: Vec<BlkxChunkEntry>, // [ num_block_chunks * blkxChunkEntry (40 bytes each)]
}

impl MishBlock {

    pub fn new_from_base_64(encoded: String) -> Result<MishBlock, &'static str> {

        // trim leading and trailing whitespace, remove all tabs and newlines
        let stripped = encoded.trim().replace("\t", "").replace("\n", "");
        //println!("base64: {}", stripped);

        let decoded = decode(&stripped).expect("Could not decode base64 data");
        //println!("decoded: {:?}", decoded);

        MishBlock::new(decoded)
    }

    pub fn new(buffer: Vec<u8>) -> Result<MishBlock, &'static str> {

        let magic = util::read_be_u32(&mut & buffer[0..4]);
        assert_eq!(format!("{:#X}", magic), MISH_MAGIC);

        // work out the number of block chunks now, as we'll reuse it
        let number_block_chunks: u32 = util::read_be_u32(&mut &buffer[200..204]);

        Ok(MishBlock {
            signature: util::read_be_u32(&mut &buffer[0..4]),
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
                data: util::from_buffer(&buffer[72..200]),
            },

            number_block_chunks,
            block_entries: MishBlock::build_block_entries(
                buffer[204..].to_vec(), 
                number_block_chunks as usize),
        })
    }

    fn build_block_entries(buffer: Vec<u8>, count: usize) -> Vec<BlkxChunkEntry> {
        
        let mut entries: Vec<BlkxChunkEntry> = Vec::with_capacity(count);

        let mut chunks = buffer.chunks_exact(BLKX_CHUNK_ENTRY_SIZE);

        while entries.len() < count {
            let c = chunks.next().unwrap().to_vec();
            let entry = BlkxChunkEntry::new(c).unwrap();
            entries.push(entry);
        }

        entries
    }
}


// Can just list partitions for now
// Needs Error handling
#[derive(Debug)]
pub struct PList {
    partitions: Vec<PartitionEntry>,
}

// Parse the XML plist data 
// Needs proper Error type
pub fn parse_plist(data: Vec<u8>) -> Result<PList, xmltree::ParseError> {

    let string = String::from_utf8(data).unwrap();
    //println!("xml: {}", string);

    let xml = xmltree::Element::parse(string.as_bytes())?;

    //println!("{:#?}", xml.children);

    let outer_dict = xml.get_child("dict").unwrap();
    //println!("children: {:#?}", outer_dict);

    // check for the resource-fork key
    let resource_fork = outer_dict.get_child("key").expect("Could not find resource-fork");
    let text = resource_fork.text.clone().expect("Malformed resource-fork text");
    assert_eq!(text, "resource-fork");

    // get the array that contains the blk data entries
    let blk_array = outer_dict.get_child("dict").unwrap()
                        .get_child("array").expect("Could not find blk data array");

    //println!("blk array {:#?}", blk_array);

    let partitions: Vec<PartitionEntry> = blk_array
    .children
    .iter()
    .map(|child| PartitionEntry::new(child.clone()).unwrap())
    .collect();

    //println!("Found partitions: {:#?}", partitions);

    Ok(PList {
        partitions,
    })
}
