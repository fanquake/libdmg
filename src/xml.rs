use std::io::Read;
use std::error;
use std::fmt;
use xmltree;

use crate::partition::PartitionEntry;

/// Ways that XML parsing might fail
#[derive(Debug)]
pub enum XMLError {
    Base64(String),
    Blxx(String),
    Mish(String),
    Partition(String),
    XML(String),
}

impl fmt::Display for XMLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XMLError::Base64(e) => fmt::Display::fmt(e, f),
            XMLError::Blxx(e) => fmt::Display::fmt(e, f),
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
            XMLError::Blxx(e) => e,
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
pub struct PList {
    /// Vector of GPT partitions
    pub partitions: Vec<PartitionEntry>,
}

impl PList {

    pub fn from_bytes(data: Vec<u8>) -> Result<PList, XMLError> {
        let string = String::from_utf8(data).unwrap();
        //println!("xml: {:#?}", string.clone());

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

        //println!("out_dict: {:#?}", outer_dict);

        // get the array that contains the blk data entries
        let blk_array = outer_dict
            .get_child("dict")
            .unwrap()
            .get_child("array")
            .expect("Could not find blk data array");

    //println!("blk array: {:#?}", blk_array);

    //    let plst_array = outer_dict
    //         .get_child("dict")

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

    // Create an empty XML structure that looks something
    // like the plist data we want to end up with
    fn empty() -> xmltree::Element {

        // This may also need Csum and Nsiz keys,
        // but they are not present in the hdiutil generated DMG
        let base_xml: &'static str = r##"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
<key>resource-fork</key>
<dict>
<key>blkx</key><array></array>
<key>plst</key><array></array>
</dict>
</dict></plist>
"##;

        xmltree::Element::parse(base_xml.as_bytes()).unwrap()
    }

    // Should not be affected by BE ordering. All data being passed in 
    // has already been converted to BE bytes
    pub fn build(mish: String) -> Vec<u8> {
        let mut base = PList::empty();
        let partition = PList::partition(mish, 0);

        // insert our new partition
        let blk_array = base.get_mut_child("dict")
                            .unwrap().get_mut_child("dict")
                            .unwrap().get_mut_child("array").unwrap();
        blk_array.children.push(partition);

        println!("Built XML: {:#?}", base);

        base.write(std::fs::File::create("plist.xml").unwrap()).unwrap();
        let mut contents = Vec::new();
        let mut ret = std::fs::File::open("plist.xml").unwrap();
        ret.read_to_end(&mut contents).unwrap();

        contents
    }

    fn partition(mish: String, partition_id: u32) -> xmltree::Element {
        // partition dictionary
        let mut part = xmltree::Element::new("dict");

        // Attributes
        let attr = PList::component(ElementType::KeyElm, String::from("Attributes"));
        part.children.push(attr);
        let attr_value = PList::component(ElementType::StringElm, String::from("0x0050"));
        part.children.push(attr_value);

        // Data
        let data = PList::component(ElementType::KeyElm, String::from("Data"));
        part.children.push(data);
        let data_val = PList::component(ElementType::DataElm, mish);
        part.children.push(data_val);
        // ID
        let id = PList::component(ElementType::KeyElm, String::from("ID"));
        part.children.push(id);
        let id_val = PList::component(ElementType::StringElm, partition_id.to_string());
        part.children.push(id_val);
        // Name
        let name = PList::component(ElementType::KeyElm, String::from("Name"));
        part.children.push(name);
        let name_val = PList::component(ElementType::StringElm, String::from("whole disk (unknown partition : 0)"));
        part.children.push(name_val);
        // CFName
        let cf = PList::component(ElementType::KeyElm, String::from("CFName"));
        part.children.push(cf);
        let cf_val = PList::component(ElementType::StringElm, String::from("whole disk (unknown partition : 0)"));
        part.children.push(cf_val);

        part
    }

    fn component(element_type: ElementType, text: String) -> xmltree::Element {
        let mut c = xmltree::Element::new(element_type.to_str());
        c.text = Some(text);
        c
    }
}

pub enum ElementType {
    ArrayElm,
    DictElm,
    DataElm,
    KeyElm,
    StringElm,
}

impl ElementType {
    pub fn to_str(self) -> &'static str {
        match self {
            ElementType::ArrayElm => "array",
            ElementType::DataElm => "data",
            ElementType::DictElm => "dict",
            ElementType::KeyElm => "key",
            ElementType::StringElm => "string",
        }
    }
}
