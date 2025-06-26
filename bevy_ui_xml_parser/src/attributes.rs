use regex::Regex;
use crate::into::{Attribute, NodeValue};
use crate::{XmlAttribute, XmlTag};
use crate::error::XmlLayoutError;
use crate::layout_reader::LayoutReader;

fn parse_args(args_str: &str) -> Vec<String> {
    args_str
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_attribute_value(input: &str) -> Result<NodeValue, XmlLayoutError> {
    let binding = Regex::new(r"\{Binding\s+([A-Za-z_][A-Za-z0-9_]*)}").unwrap();
    let local = Regex::new(r"\{LocalResource\s+([A-Za-z_][A-Za-z0-9_]*)}").unwrap();
    let global = Regex::new(r"\{GlobalResource\s+([A-Za-z_][A-Za-z0-9_]*)}").unwrap();
    let function = Regex::new(r"^@([A-Za-z_][A-Za-z0-9_]*)\(([^()]*)\)$").unwrap();

    // Function
    if let Some(caps) = function.captures(&input) {
        let name = caps.get(1).unwrap().as_str();
        let args = parse_args(caps.get(2).map_or("", |m| m.as_str()));
        return Ok(NodeValue::CallFunction {
            args,
            name: name.to_string(),
        });
    }

    //Binding
    if let Some(caps) = binding.captures(&input) {
        let value = caps.get(1).unwrap().as_str().to_string();
        return Ok(NodeValue::Binding(value));
    }

    //Local res
    if let Some(caps) = local.captures(&input) {
        let value = caps.get(1).unwrap().as_str().to_string();
        return Ok(NodeValue::Local(value));
    }

    //Global res
    if let Some(caps) = global.captures(&input) {
        let value = caps.get(1).unwrap().as_str().to_string();
        return Ok(NodeValue::Global(value));
    }

    // Default: Value
    Ok(NodeValue::Value(input.to_string()))
}

pub(super) fn parse_attributes(vec: Vec<XmlAttribute>) -> Result<Vec<Attribute>, XmlLayoutError> {
    let mut result = Vec::new();
    for attr in vec.into_iter() {
        let value = parse_attribute_value(&attr.value())?;
        result.push(Attribute {
            value,
            name: attr.into_name(),
        });
    }

    Ok(result)
}

impl<'a> LayoutReader<'a> {
    pub(super) fn parse_required_attribute(&self, tag: &XmlTag, attribute: &'static str) -> Result<String, XmlLayoutError> {
        Ok(tag.attributes()
            .iter()
            .find(|&a| a.name() == attribute)
            .ok_or(self.err_missing_attribute(tag, attribute))?
            .value()
            .to_string())
    }
}