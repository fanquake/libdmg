
use std::fs::{File};
use std::io::{self};
use std::io::{prelude::{Read}};
use std::io::Write;

use super::blkx::{BlkxChunkEntry, DmgBlxx};
use super::koly::KolyBlock;
use super::mish::MishBlock;
use super::util::UDIFChecksum;
use super::xml::PList;

use base64::{encode};
use libflate::deflate::{Encoder};

/// Mimics the behaviour of libdmg-hfsplus compress function
/// Takes the ISO generated by genisoimage (containing Bitcoin-Core.app) and
/// converts it into a "proper" DMG image.
pub fn conversion(iso: std::path::PathBuf, dmg: std::path::PathBuf) -> Result<(), io::Error> {
    println!("converting: {:#?}, to dmg: {:#?}", iso, dmg);

    let mut f = File::open(iso)?;
    //println!("metadata: {:#?}", f.metadata()?);

    // read the entire incoming ISO image into a buffer.
    let mut incoming = Vec::new();
    f.read_to_end(&mut incoming)?;
    let incoming_size = incoming.len();
    println!("Incoming ISO size: {:#?}", incoming_size);

    // build the block entries
    // zlib deflate in 512 byte chunks

    // number of sectors required is incoming_size / 512
    let mut sectors_required = incoming_size / 512;
    println!("sectors required: {:#?}", sectors_required);

    let mut entries: Vec<BlkxChunkEntry> = Vec::new();

    let mut current_run = 0; // TODO - can be removed
    let mut sectors_processed = 0;

    // buffer that zlib compressed data is being written into
    let mut processed_buffer: Vec<u8> = Vec::new();


    while sectors_required > 0 {
        let sector_count = if sectors_required > 512 { 512 } else { sectors_required };
        println!("start of run {}: sectors={}, left={}, processed: {}", current_run, sector_count, sectors_required, sectors_processed);

        let start = sectors_processed * 512;
        let end = (sectors_processed + sector_count) * 512;

        // zlib compress bytes, and put them into processed buffer
        let mut encoder = Encoder::new(Vec::new());
        io::copy(&mut &incoming[start..end], &mut encoder).unwrap();
        let encoded = encoder.finish().into_result().unwrap();
        processed_buffer.append(&mut encoded.clone());

        // add the number of sectors just processed
        sectors_processed += sector_count;

        let compressed_length = encoded.len() as u64;

        // build a BlkxChunkEntry
        let entry = BlkxChunkEntry {
            entry_type: DmgBlxx::ZLibCompression,
            comment: 0,
            sector_number: (sectors_processed - sector_count) as u64,
            sector_count: sector_count as u64,
            compressed_offset: (processed_buffer.len() as u64) - compressed_length,
            compressed_length,
        };

        entries.push(entry);

        sectors_required -= sector_count;
        current_run += 1;
    };

    // build the final blkx chunk entry
    let final_entry = final_blkx(incoming_size / 512, processed_buffer.len());
    entries.push(final_entry);

    println!("entries: {:#?}", entries.len());
    //println!("built entries: {:#?}", entries);
    println!("Original buffer was: {:#?}", incoming_size);
    // 15148687 default usage
    // TODO: Need to investigate using Best compression level
    println!("zlib buffer size: {:#?}", processed_buffer.len());

    // Now we've got blkx chunk entries, build a mish block
    let mish_block = build_mish((incoming_size / 512) as u64, entries);
    println!("{:#?}", mish_block);

    let be_mish_bytes = mish_block.to_be_bytes();

    let encoded = encode(&be_mish_bytes);
    //println!("base64: {:#?}", encoded);

    // construct the XML plist data, including the mish block
    let mut xml = PList::build(encoded);
    //println!("xml: {:#?}", String::from_utf8(xml.clone()));

    // generate the koly block
    let koly = build_koly(xml.len() as u64, processed_buffer.len() as u64, (incoming_size / 512) as u64);

    // append the XML to the processed buffer
    processed_buffer.append(&mut xml);

    // serialize and append the koly block to the end of the buffer
    let mut koly_be = koly.to_be_bytes();
    processed_buffer.append(&mut koly_be);

    // write out the progressed buffer to disk
    let mut result = std::fs::File::create(dmg).unwrap();
    result.write_all(&processed_buffer).unwrap();

    Ok(())
}

pub fn build_koly(xml_length: u64, data_fork_length: u64, sector_count: u64) -> KolyBlock {
    KolyBlock {
        magic: 1_802_464_377,
        version: 4,
        header_size: 512,
        flags: 1,
        running_data_fork_offset: 0,
        data_fork_offset: 0,
        data_fork_length,
        source_fork_offset: 0,
        source_fork_length: 0,
        segment_number: 0, // TODO - 1 ?
        segment_count: 0, // TODO - 1 ?
        segment_id: 0, // GUID
        data_fork_checksum: UDIFChecksum {
            fork_type: 0,
            size: 0,
            data: vec![0u8; 128],
        },
        xml_offset: data_fork_length,
        xml_length,
        reserved_one: vec![0u8; 120],
        master_checksum: UDIFChecksum {
            fork_type: 2,
            size: 32,
            data: vec![0u8; 128], // TODO - actually calculate and set this
        },
        image_variant: 2,
        sector_count, // Need to check if this is actually correct
        reserved_two: 0,
        reserved_three: 0,
        reserved_four: 0,
    }
}

pub fn build_mish(sectors: u64, entries: Vec<BlkxChunkEntry>) -> MishBlock {
    MishBlock {
        signature: 1_835_627_368,
        version: 1,
        sector_number: 0,
        sector_count: sectors,
        data_offset: 0,
        buffers_needed: 520, // from libdmg-hfsplus
        block_descriptors: 4_294_967_294,
        reserved_1: 0,
        reserved_2: 0,
        reserved_3: 0,
        reserved_4: 0,
        reserved_5: 0,
        reserved_6: 0,
        checksum: UDIFChecksum {
            fork_type: 2,
            size: 32,
            data: vec![0u8; 128], // TODO - actually calculate and set
        },
        number_block_chunks: (entries.len() as u32),
        block_entries: entries
    }
}

pub fn final_blkx(sectors: usize, offset: usize) -> BlkxChunkEntry {
    BlkxChunkEntry {
        entry_type: DmgBlxx::LastEntry,
        comment: 0,
        sector_count: 0,
        sector_number: sectors as u64,
        compressed_length: 0,
        compressed_offset: offset as u64
    }
}
