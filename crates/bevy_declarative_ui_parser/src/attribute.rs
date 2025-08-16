use crate::errors::XmlLayoutError;
use crate::layout_reader::LayoutReader;
use crate::lexer::Value;
use crate::values::AttributeValue;
use crate::{XmlTag, lexer};

#[derive(PartialEq, Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

pub(super) fn parse_attributes(
    reader: &LayoutReader,
    vec: Vec<lexer::Attribute>,
) -> Result<Vec<Attribute>, XmlLayoutError> {
    let mut result = Vec::with_capacity(vec.len());
    vec.into_iter()
        .try_for_each(|attr| -> Result<(), XmlLayoutError> {
            result.push(Attribute {
                value: AttributeValue::parse(reader, &attr.value, false)?,
                name: attr.into_name(),
            });
            Ok(())
        })?;

    Ok(result)
}

impl<'a> LayoutReader<'a> {
    pub(super) fn parse_required_attribute(
        &self,
        tag: &XmlTag,
        attribute: &'static str,
    ) -> Result<Value, XmlLayoutError> {
        Ok(tag
            .attributes()
            .iter()
            .find(|&a| a.name() == attribute)
            .ok_or(self.err_missing_attribute(tag, attribute))?
            .value
            .clone())
    }
}
