use xml_parser::{Token};
use crate::{UiNode, XmlTag};
use crate::error::XmlLayoutError;
use crate::into::NodeValue;
use crate::layout_reader::LayoutReader;

impl<'a> LayoutReader<'a> {
    pub(super) fn parse_container(&mut self, tag: XmlTag) -> Result<UiNode, XmlLayoutError> {
        let mut node: UiNode = UiNode::container(tag.clone())?;

        for attribute in &node.tag.attributes {
            if attribute.name == "id" && !matches!(attribute.value, NodeValue::Value(_)) {
                return Err(self.err_expected_value(&tag));
            }
        }

        loop {
            match self.inner.read()? {
                Token::TagStart(tag) => {
                    if tag.identifier() != "Container" {
                        return Err(self.err_unexpected_tag(tag, vec!["Container", "Any component"]));
                    }
                    node.children.push(self.parse_container(tag)?);
                }
                Token::TagEmpty(tag) => node.children.push(UiNode::component(tag)?),
                Token::TagEnd(_) => return Ok(node),
                Token::EOF => return Err(self.err_end_of_file()),
                _ => {}
            }
        }
    }
}