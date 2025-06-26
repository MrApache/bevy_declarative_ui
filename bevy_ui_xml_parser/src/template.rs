use std::collections::HashSet;
use xml_parser::{Token};
use crate::{resources::{
    Resource,
    Resources
}, UiNode, XmlTag};
use crate::error::XmlLayoutError;
use crate::layout_reader::LayoutReader;

#[derive(Default, Debug)]
pub struct Template {
    pub name: String,
    pub nodes: Vec<UiNode>,
    pub containers: HashSet<String>,
    pub resources: Resources,
}

impl Template {
    pub fn new(reader: &LayoutReader, tag: &XmlTag) -> Result<Self, XmlLayoutError> {
        let name = reader.parse_required_attribute(&tag, "name")?;
        if name.is_empty() {
            return Err(reader.err_empty_attribute(tag, "name"));
        }
        let container = reader.parse_required_attribute(&tag, "container")?;
        let list: HashSet<String> = container
            .trim()
            .split(';')
            .map(|s| s.trim().to_string())
            .collect();

        let mut template: Template = Template::default();
        template.name = name;
        template.containers = list;

        Ok(template)
    }
}

impl<'a> LayoutReader<'a> {
    pub(super) fn parse_template(&mut self, tag: XmlTag) -> Result<Template, XmlLayoutError> {
        let mut template = Template::new(&self, &tag)?;
        loop {
            match self.inner.read()? {
                Token::TagEmpty(tag) => template.nodes.push(UiNode::component(tag)?),
                Token::TagStart(tag) => {
                    match tag.identifier() {
                        "Container" => template.nodes.push(self.parse_container(tag)?),
                        "LocalResources" => self.parse_resources(&mut template.resources, Resource::Local)?,
                        _ => return Err(self.err_unexpected_tag(tag, vec!["Container", "LocalResources"])),
                    }
                }
                Token::TagEnd(tag) => {
                    match tag.identifier() {
                        "Container" => {},
                        "Template" => {
                            if template.nodes.len() == 0 {
                                return Err(XmlLayoutError::EmptyTemplate);
                            }
                            return Ok(template);
                        },
                        _ => panic!()
                    }
                }
                Token::EOF => return Err(self.err_end_of_file()),
                _ => {}
            }
        }
    }
}