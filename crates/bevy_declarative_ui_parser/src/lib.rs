pub mod into;
pub mod attribute;
pub mod values;
pub mod utils;
pub mod position;
pub mod errors;
mod template;
mod layout_reader;
mod layout_errors_impls;
mod states;
mod lexer;

pub use layout_reader::LayoutReader;
pub use template::ItemTemplate;

use std::collections::HashSet;
use crate::errors::XmlLayoutError;
use crate::into::Tag;

type XmlTag = lexer::Tag;

#[derive(Default, Debug)]
pub struct XmlLayout {
    pub templates:  Vec<ItemTemplate>,
    pub root_nodes: Vec<UiNode>,
    pub usings:     HashSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Id {
    Default(u64),
    Template(u64),
    Custom(String),
    Runtime(String),
}

impl Id {
    pub fn to_runtime(self) -> Self {
        if let Self::Runtime(_) = self {
            unreachable!()
        }
        else {
            Self::Runtime(format!("Runtime{self}"))
        }
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Default(number) => write!(f, "Id{number}"),
            Id::Template(number) => write!(f, "Template{number}"),
            Id::Custom(identifier) => write!(f, "{identifier}"),
            Id::Runtime(identifier) => write!(f, "{identifier}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiNode {
    pub tag: Tag,
    pub id:  Id,
    pub components: Vec<Tag>,
    pub children:   Vec<UiNode>,
}

impl UiNode {
    pub fn new(reader: &LayoutReader, tag: XmlTag) -> Result<UiNode, XmlLayoutError> {
        Ok(UiNode {
            tag:  Tag::from(reader, tag)?,
            id:   Id::Default(0),
            components: vec![],
            children:   vec![],
        })
    }
}