use std::fs::{File};
use std::io;
use std::io::{prelude::{Read, Seek}, SeekFrom};
use structopt::StructOpt;

extern crate base64;
extern crate libflate;
extern crate xmltree;

pub mod blkx;
pub mod convert;
pub mod koly;
pub mod mish;
pub mod partition;
pub mod util;
pub mod xml;

use convert::*;
use koly::*;
use xml::*;

#[derive(StructOpt)]
#[structopt(name = "libdmg", about = "DMG inspection and creation")]
enum Cli {
    #[structopt(name = "inspect")]
    /// Inspect a DMG file and print metadata
    Inspect {
        /// path to a DMG file
        file: std::path::PathBuf,
    },
    #[structopt(name = "convert")]
    /// Create a DMG file from the given folder
    Convert {
        /// path to an ISO image (generate by genisoimage)
        iso: std::path::PathBuf,
        /// where to create the DMG
        dmg: std::path::PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    match args {
        Cli::Inspect { file } => inspect(&file)?,
        Cli::Convert { iso, dmg } => conversion(iso, dmg)?,
    }

    Ok(())
}

fn inspect(file: &std::path::PathBuf) -> Result<(), io::Error> {

    // Open the file, and dump some metadata
    let mut f = File::open(file)?;
    println!("Inspecting: {:#?}", file.file_name().expect("Could not retrieve file name.."));

    // 13741856
    //println!("{:#?}", f.metadata()?.len());

    // Seek -512 bytes from the end of the file,
    // this should be the start of the "koly block"
    f.seek(SeekFrom::End(-512))?;

    let mut koly_block = Vec::new();
    f.read_to_end(&mut koly_block)?;

    let udif_res = KolyBlock::new(koly_block).expect("Could not read koly block!");
    println!("udif: {:#?}", udif_res);

    // Once we've parsed the koly block, we can grab the xml length and offset
    // and use that to extract and parse the plist xml.
    f.seek(SeekFrom::Start(udif_res.xml_offset))?;

    // 10667
    println!("{:#?}", &udif_res.xml_length);

    let mut plist = vec![0u8; udif_res.xml_length as usize];
    f.read_exact(&mut plist)?;

    let _parsed = PList::from_bytes(plist).unwrap();

    println!("parsed: {:#?}", _parsed);

    //println!("chunk 0: {:#?}", parsed.partitions[0]);
    // println!("chunk 1: {:#?}", parsed.partitions[0].data.block_entries[1]);
    // println!("chunk 2: {:#?}", parsed.partitions[0].data.block_entries[2]);

    // let names: Vec<String> = parsed.partitions.into_iter().map(|p| p.name).collect();
    // println!("names: {:#?}", names);

    // data fork length - 13730667
    // xml offset - 13730667
    // xml length - 10677


    // let core = &parsed.partitions[0];
    // println!("{:#?}", core);

    // f.seek(SeekFrom::Start(0))?;
    // let mut encoded = vec![0u8; 13730667 as usize];

    // f.read_exact(&mut encoded)?;

   // let decoded = decode_zlib_block(encoded);

    // 262144
    //println!("{:#?}", decoded.len());
    // 77, 97, 99, 79, 83 - MacOS
    // 66, 105, 116, 99, 111, 105, 110, 45, 81, 116 - Bitcoin-Qt
    // let bitcoin = [77, 97, 99, 79, 83].to_vec();
    // let length = bitcoin.len();
    // let found = find(decoded.clone(), bitcoin);

    // println!("Found index: {:#?}", found);
    // println!("bitcoin: {:#?}", decoded[found..found+length].to_vec());
    // //println!("len: {}", &filtered.len());
    // println!("decoded len: {}", decoded.len());

    Ok(())
}

// fn find(search_in: Vec<u8>, for_bytes: Vec<u8>) -> usize {
//     search_in.windows(for_bytes.len()).position(|x| x.to_vec() == for_bytes).unwrap()
// }

// fn decode_zlib_block(encoded: Vec<u8>) -> Vec<u8> {

//     let mut decoder = Decoder::new(&encoded[..]).unwrap();
//     let mut decoded = Vec::new();
//     decoder.read_to_end(&mut decoded).unwrap();
//     decoded
// }

    // HELLO WORLD SEARCHING
    // let apfs = &parsed.partitions[4];
    // println!("APFS: {:#?}", apfs);

    // // Lets go searching for the compressed "Hello World" !
    // f.seek(SeekFrom::Start(472))?;
    // let mut encoded_data = vec![0u8; 5007 as usize];
    // f.read_exact(&mut encoded_data)?;

    // let decoded_data = decode_zlib_block(encoded_data);

    // //let filtered: Vec<u8> = decoded_data.to_vec().into_iter().filter(|e| e > &0u8).collect();
    // //println!("unzlibbed: {:#?}", filtered);

    // let hello_world = [72, 101, 108, 108, 111, 32, 87, 111, 114, 108].to_vec();
    // let length = hello_world.len();
    // let found = find(decoded_data.clone(), hello_world);

    // println!("Found index: {:#?}", found);
    // println!("Hello world: {:#?}", decoded_data[found..found+length].to_vec());
    // //println!("len: {}", &filtered.len());
    // println!("decoded len: {}", decoded_data.len());

    // Ok(())