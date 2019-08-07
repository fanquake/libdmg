use super::xml::XMLError;
use super::mish::MishBlock;

/// Describes a GPT partition
#[derive(Debug)]
pub struct PartitionEntry {
    /// Some attributes as a hex string. Generally 0x0050 ?
    pub attributes: String,
    /// Column family name?
    pub cf_name: String,
    /// Base64 encoded string, decoded into a mish block
    pub data: MishBlock,
    /// Id in the range -1...number of partition entries
    pub id: i32,
    /// Always seems to be the same as cf_name
    pub name: String,
}

impl PartitionEntry {
    pub fn new(element: &xmltree::Element) -> Result<PartitionEntry, XMLError> {
        let children = &element.children;

        // TODO: extract strings and turn static?
        let attributes = PartitionEntry::find_index_for(String::from("Attributes"), &children)?;
        let cf_name = String::from("whatever"); // PartitionEntry::find_index_for(String::from("CFName"), &children)?;
        let data = PartitionEntry::find_index_for(String::from("Data"), &children)?;
        // TODO: yuck?
        let id: i32 = PartitionEntry::find_index_for(String::from("ID"), &children)?
            .parse()
            .unwrap();
        let name = PartitionEntry::find_index_for(String::from("Name"), &children)?;

        Ok(PartitionEntry {
            attributes,
            cf_name,
            data: MishBlock::from_base64(data)?,
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