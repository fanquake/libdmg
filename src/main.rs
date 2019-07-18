use std::fs::File;
use std::io;
use std::io::{prelude::{Read, Seek}, SeekFrom};
use structopt::StructOpt;

extern crate base64;
extern crate xmltree;

mod koly;
use koly::*;

mod xml;
use xml::*;

mod util;

#[derive(StructOpt, Debug)]
#[structopt(name = "libdmg_rust", about = "DMG inspection and creation")]
enum Cli {
    #[structopt(name = "inspect")]
    /// Inspect a DMG file and print metadata
    Inspect {
        /// path to a DMG file
        file: std::path::PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    match args {
        Cli::Inspect { file } => inspect(&file)?,
    }

    Ok(())
}

fn inspect(file: &std::path::PathBuf) -> Result<(), io::Error> {

    // Open the file, and dump some metadata
    let mut f = File::open(file)?;
    println!("Inspecting: {:#?}", file.file_name().expect("Could not retrieve file name.."));

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

    let mut plist = vec![0u8; udif_res.xml_length as usize];
    f.read_exact(&mut plist)?;

    let _parsed = parse_plist(plist).expect("Could not parse XML plist!");

    Ok(())
}
