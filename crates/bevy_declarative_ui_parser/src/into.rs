use crate::attribute::{parse_attributes, Attribute};
use crate::errors::XmlLayoutError;
use crate::{lexer, LayoutReader};

#[derive(PartialEq, Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub attributes: Vec<Attribute>,
}

impl Tag {
    pub fn from(reader: &LayoutReader, xml_tag: lexer::Tag) -> Result<Tag, XmlLayoutError> {
        let (name, attributes) = xml_tag.into_inner();
        Ok(Tag {
            name,
            attributes: parse_attributes(reader, attributes)?,
        })
    }
}