mod mode;
pub mod params;
pub mod filter;
mod raw_binding;
mod kind;

pub use mode::BindingMode;
pub use kind::BindingKind;

use std::collections::HashMap;
use std::fmt::Debug;
use regex::Regex;
use raw_binding::RawBinding;
use crate::{LayoutReader, XmlLayoutError};
use crate::position::{Location, Span};
use crate::lexer::Value;
use crate::utils::{GetOrInsertEmpty, TrimExtension};
use crate::values::bindings::params::Params;

#[derive(Clone, Debug, PartialEq)]
pub struct Binding<B, A>
where
    B: Clone + Debug + PartialEq + Params,
    A: Clone + Debug + PartialEq + Params,
{
    pub base_params: B,
    pub additional_params: A,
    pub kind: BindingKind,
}

impl<B, A> Binding<B, A>
where
    B: Clone + Debug + PartialEq + Params,
    A: Clone + Debug + PartialEq + Params,
{
    pub const fn new(base: B, additional: A, kind: BindingKind) -> Self {
        Self {
            base_params: base,
            additional_params: additional,
            kind,
        }
    }
}

#[derive(Debug)]
pub struct NamedParameter {
    pub name:  Value,
    pub value: Value,
    pub span:  Span,
    pub location: Location,
}

impl NamedParameter {
    pub const fn new(name: Value, value: Value, span: Span, location: Location) -> Self {
        Self {name, value, span, location }
    }
}

impl<B, A> Binding<B, A>
where
    B: Clone + Debug + PartialEq + Params,
    A: Clone + Debug + PartialEq + Params,
{
    pub fn parse(reader: &LayoutReader, source: &Value, target: Value, params: &str) -> Result<Self, XmlLayoutError> {
        let diff = source.value().len() - params.len() - 1;

        let mut location = source.location;
        let mut params_span = source.span;
        location.column += diff;
        params_span.start += diff;
        params_span.end -= 1; //Skip '}'

        let mut unnamed = None;
        let mut named = HashMap::<String, Vec<NamedParameter>>::new();

        params.split(',').enumerate().for_each(|(i, raw)| {
            let trim_result = raw.trim_ext();
            let param = trim_result.string;
            location.column += trim_result.before;
            params_span.start += trim_result.before;

            if param.contains('=') {
                let (key, value) = param.split_once('=').unwrap();

                let key_new = key.trim_end();
                params_span.end = params_span.start + key_new.len();
                let key_result = Value::new(params_span, location, key_new);
                location.column += key_new.len();

                let value_new = value.trim_start();
                let offset = key.len() - key_new.len() + value.len() - value_new.len() + 1;
                location.column += offset;
                params_span.start = params_span.end + offset;
                params_span.end = params_span.start + value_new.len();
                let value_result = Value::new(params_span, location, value_new);

                let parameter = NamedParameter::new(key_result, value_result, Span::new(0, 0), Location::new(0 ,0, 0));
                named.get_or_insert(key_new, || vec![]).push(parameter);

                let offset = value.len() - value_new.len() + 1; //Skip whitespaces with ','
                params_span.start = params_span.end + offset;
                location.column += value_new.len() + offset; // Skip "value[whitespaces],"
            }
            else if i == 0 {
                unnamed = if param.is_empty() {
                    None
                } else {
                    params_span.end = params_span.start + param.len();
                    let value = Value::new(params_span, location, reader.substring_other(&params_span));
                    params_span.start = params_span.end + 1;
                    location.column += param.len() + 1;
                    Some(value)
                };
            }
            else {
                panic!("Warning: ignored unexpected unnamed parameter '{param}'");
            }
        });

        create_binding(reader, RawBinding::new(reader, source.clone(), target, unnamed, named)?)
    }
}

fn create_binding<B, A>(reader: &LayoutReader, mut raw: RawBinding<B>) -> Result<Binding<B, A>, XmlLayoutError>
where
    B: Clone + Debug + PartialEq + Params,
    A: Clone + Debug + PartialEq + Params,
{
    let base_params = B::read(reader, &mut raw)?;
    let additional_params = A::read(reader, &mut raw)?;
    let binding = match raw.target.value() {
        "Component" => Binding::new(base_params, additional_params, BindingKind::Component),
        "Resource"  => Binding::new(base_params, additional_params, BindingKind::Resource),
        "Item"      => Binding::new(base_params, additional_params, BindingKind::Item),
        _ => panic!("Unknown binding type: {}", raw.target.value())
    };

    Ok(binding)
}

fn format_path(input: &str) -> String {
    let array_regex = Regex::new(r"\[([a-zA-Z0-9:_]+)]").unwrap();
    array_regex.replace_all(input, |caps: &regex::Captures| {
        format!("get({}).unwrap()", &caps[1])
    }).to_string()
}