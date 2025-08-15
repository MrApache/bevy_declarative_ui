use std::fmt::Debug;
use std::marker::PhantomData;
use std::collections::HashMap;
use crate::errors::XmlLayoutError;
use crate::LayoutReader;
use crate::lexer::Value;
use crate::utils::GetOrInsertEmpty;
use crate::values::bindings::NamedParameter;
use crate::values::bindings::params::Params;

pub struct RawBinding<B>
where
    B: Clone + Debug + PartialEq + Params,
{
    pub source: Value,
    pub target: Value,
    pub params: Vec<NamedParameter>,
    _marker: PhantomData<B>,
}

impl<B> RawBinding<B>
where
    B: Clone + Debug + PartialEq + Params,
{
    pub fn new(
        reader: &LayoutReader,
        source: Value,
        target: Value,
        unnamed: Option<Value>,
        mut named: HashMap<String, Vec<NamedParameter>>
    ) -> Result<Self, XmlLayoutError> {

        let mut result = Vec::new();
        if let Some(unnamed) = unnamed && let Some(shortcut) = B::unnamed_param() {
            let mut span = unnamed.span;
            span.end = span.start + unnamed.inner.len();
            let location = unnamed.location;
            let fake_name = Value::new(span, location, shortcut);
            named.get_or_insert(shortcut, ||vec![]).push(NamedParameter::new(fake_name, unnamed, span, location));
        }

        for (key_name, values) in named {
            if values.len() > 1 {
                let values = values.into_iter().map(|v| v.name).collect::<Vec<Value>>();
                return Err(reader.err_duplicate_param(&source, &values, key_name.as_str()));
            }
            result.extend(values);
        }

        Ok(Self {
            source,
            target,
            params: result,
            _marker: PhantomData::default(),
        })
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.params.iter()
            .find(|p| p.name.value().eq(key))
            .is_some()
    }

    pub fn get_duplicates(&self, key: &str) -> Vec<Value> {
        self.params.iter()
            .filter(|p| p.name.value().eq(key))
            .map(|p| p.value.clone())
            .collect()
    }

    pub fn try_take(&mut self, key: &str) -> Option<NamedParameter> {
        if let Some(pos) = self.params.iter().position(|p| p.name.value().eq(key)) {
            return Some(self.params.remove(pos));
        }

        None
    }

    pub fn try_take_value(&mut self, key: &str) -> Option<String> {
        if let Some(pos) = self.params.iter().position(|p| p.name.value().eq(key)) {
            let value = self.params.remove(pos);
            return Some(value.value.inner);
        }

        None
    }
}