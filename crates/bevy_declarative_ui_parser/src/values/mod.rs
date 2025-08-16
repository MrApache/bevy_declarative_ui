mod asset;
pub mod bindings;
mod function;
mod item;

pub use asset::Asset;
pub use function::Function;
pub use item::Item;

use std::collections::HashMap;

use crate::lexer::Value;
use crate::utils::IsCurlyBracesEnclosed;
use crate::utils::TrimExtension;
use crate::values::bindings::Binding;
use crate::values::bindings::params::ComponentParams;
use crate::values::bindings::params::ItemBaseParams;
use crate::values::bindings::params::{AdditionalParams, BaseParams};
use crate::{LayoutReader, XmlLayoutError};

#[derive(Clone, PartialEq, Debug)]
pub enum TemplateBinding {
    Resource(Binding<BaseParams, ()>),
    Component(Binding<BaseParams, ComponentParams>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum AttributeValue {
    Value(String),
    Asset(Asset),
    Item(Binding<ItemBaseParams, AdditionalParams>),
    Resource(Binding<BaseParams, AdditionalParams>),
    Component(Binding<BaseParams, ComponentParams>),
    Template(TemplateBinding),
}

impl AttributeValue {
    pub fn parse(
        reader: &LayoutReader,
        value: &Value,
        template: bool,
    ) -> Result<Self, XmlLayoutError> {
        let input = &value.inner;
        if input.is_curly_braces_enclosed() {
            let trim_result = input[1..input.len() - 1].trim_ext();
            let unwrap_input = trim_result.string;
            let (target, params) = unwrap_input
                .split_once(char::is_whitespace)
                .unwrap_or((unwrap_input, ""));

            let mut location = value.location;
            let mut target_span = value.span;
            location.column += trim_result.before + 1;
            target_span.start += trim_result.before + 1;
            target_span.end = target_span.start + target.len();
            let target = Value::new(target_span, location, target);
            Ok(match target.inner.as_str() {
                "Asset" => AttributeValue::Asset(Asset::parse(params)),
                "Item" => AttributeValue::Item(Binding::parse(reader, value, target, params)?),
                "Component" if !template => {
                    AttributeValue::Component(Binding::parse(reader, value, target, params)?)
                }
                "Resource" if !template => {
                    AttributeValue::Resource(Binding::parse(reader, value, target, params)?)
                }
                "Component" if template => AttributeValue::Template(TemplateBinding::Component(
                    Binding::parse(reader, value, target, params)?,
                )),
                "Resource" if template => AttributeValue::Template(TemplateBinding::Resource(
                    Binding::parse(reader, value, target, params)?,
                )),
                other => panic!("The actual input is not a valid attribute value: {other}"),
            })
        } else {
            Ok(AttributeValue::Value(input.to_string()))
        }
    }
}

fn parse_params(input: &str) -> HashMap<&str, &str> {
    let mut params = HashMap::new();
    let mut start = 0;
    let mut end = 0;
    let mut inside_quotes = false;
    input.chars().into_iter().enumerate().for_each(|(i, c)| {
        let is_eol = input.len() - 1 == i;
        end += 1;

        if inside_quotes && !is_eol {
            if c == '}' {
                inside_quotes = false;
            }
        } else if is_eol {
            let param = &input[start..end];
            let (name, value) = param.split_once('=').unwrap();
            params.insert(name.trim(), value.trim());
            start = end;
        } else if c == ',' {
            end -= 1;

            let param = &input[start..end];
            let (name, value) = param.split_once('=').unwrap();
            params.insert(name.trim(), value.trim());

            end += 1;
            start = end;
        } else if c == '{' {
            inside_quotes = true;
        }
    });

    params
}
