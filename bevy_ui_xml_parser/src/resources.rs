use std::collections::HashMap;
use xml_parser::{Token};
use crate::XmlTag;
use crate::error::XmlLayoutError;
use crate::layout_reader::LayoutReader;

#[derive(Clone, Debug, Default)]
//Name, Type, Value
pub struct Resources {
    pub(super) properties: HashMap<String, PropertyValue>
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropertyValue {
    pub type_: String,
    pub value: String,
}

impl Resources {
    pub fn insert(&mut self, name: &str, type_: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(name.to_string(), PropertyValue {
            type_: type_.into(),
            value: value.into(),
        });
    }

    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PropertyValue)> {
        self.properties.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

pub(super) enum Resource {
    Local,
    Global
}

impl<'a> LayoutReader<'a> {
    pub(super) fn parse_resources(
        &mut self,
        resources: &mut Resources,
        resource:  Resource
    ) -> Result<(), XmlLayoutError>
    {
        loop {
            match self.inner.read()? {
                Token::TagStart(tag) => panic!("{}", tag.identifier()),
                Token::TagEmpty(tag) => {
                    if tag.identifier() != "Property" {
                        return Err(self.err_unexpected_tag(tag, vec!["Property"]));
                    }

                    let (key, type_, value) = self.parse_property(&tag)?;
                    resources.properties.insert(key, PropertyValue {
                        type_,
                        value,
                    });
                }
                Token::TagEnd(tag) => {
                    match resource {
                        Resource::Local => {
                            if tag.identifier() == "LocalResources" {
                                return Ok(());
                            }
                        }
                        Resource::Global => {
                            if tag.identifier() == "GlobalResources" {
                                return Ok(());
                            }
                        }
                    }
                    panic!("{}", tag.into_identifier());
                }
                Token::EOF => return Err(self.err_end_of_file()),
                _ => {}
            }
        }
    }

    fn parse_property(&self, tag: &XmlTag) -> Result<(String, String, String), XmlLayoutError>{
        let name = self.parse_required_attribute(&tag, "name")?;
        if name.is_empty() {
            return Err(self.err_empty_attribute(tag, "name"));
        }

        let type_ = self.parse_required_attribute(&tag, "type")?;
        if type_.is_empty() {
            return Err(self.err_empty_attribute(tag, "type"));
        }

        let value = tag.attributes()
            .iter()
            .find(|&a| a.name() == "value")
            .map(|a| a.value())
            .unwrap_or("")
            .to_string();

        Ok((name, type_, value))
    }
}