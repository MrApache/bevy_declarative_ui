#[cfg(test)]
mod tests;
mod container;
mod into;
mod attributes;
mod template;
mod resources;
mod error;
mod layout_reader;
mod layout_errors_impls;

pub use layout_reader::LayoutReader;
pub use into::NodeValue;
pub use resources::{Resources, PropertyValue};
pub use template::Template;
pub use error::XmlLayoutError;

use std::collections::HashSet;
use crate::into::Tag;

type XmlAttribute = xml_parser::Attribute;
type XmlTag = xml_parser::Tag;

#[derive(Default, Debug)]
pub struct XmlLayout {
    pub local:      Resources,
    pub global:     Resources,
    pub templates:  Vec<Template>,
    pub root_nodes: Vec<UiNode>,
    pub usings:     HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct UiNode {
    pub tag: Tag,
    pub children:   Vec<UiNode>,
}

impl UiNode {
    pub fn container(tag: XmlTag) -> Result<UiNode, XmlLayoutError> {
        Ok(UiNode {
            tag: Tag::from(tag, true)?,
            children: vec![],
        })
    }

    pub fn component(tag: XmlTag) -> Result<UiNode, XmlLayoutError> {
        Ok(UiNode {
            tag: Tag::from(tag, false)?,
            children: vec![],
        })
    }
}