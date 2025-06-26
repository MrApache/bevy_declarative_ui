use crate::attributes::parse_attributes;
use crate::error::XmlLayoutError;

#[derive(PartialEq, Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub is_container: bool,
}

impl Tag {
    pub fn from(xml_tag: xml_parser::Tag, is_container: bool) -> Result<Tag, XmlLayoutError> {
        let (name, attributes) = xml_tag.into_inner();
        Ok(Tag {
            name,
            is_container,
            attributes: parse_attributes(attributes)?,
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: NodeValue,
}

#[derive(Clone, PartialEq, Debug)]
pub enum NodeValue {
    Value(String),
    Property(String),
    Binding(String),
    Local(String),
    Global(String),
    CallFunction {
        name: String,
        args: Vec<String>
    },
}

impl NodeValue {
    pub fn read_value(&self) -> String {
        match self {
            NodeValue::Value(value) => value.clone(),
            _ => panic!("Todo"),
        }
    }
}
