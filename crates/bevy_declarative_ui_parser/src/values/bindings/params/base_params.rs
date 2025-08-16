use crate::values::bindings::params::Params;
use crate::values::bindings::raw_binding::RawBinding;
use crate::values::bindings::{BindingMode, format_path};
use crate::{LayoutReader, XmlLayoutError};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub struct BaseParams {
    pub target: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdditionalParams {
    pub converter: Option<String>,
    pub fallback: Option<String>,
    pub mode: BindingMode,
}

impl Params for BaseParams {
    fn unnamed_param() -> Option<&'static str> {
        Some("Target")
    }

    fn read<B>(reader: &LayoutReader, raw: &mut RawBinding<B>) -> Result<Self, XmlLayoutError>
    where
        Self: Sized,
        B: Clone + Debug + PartialEq + Params,
    {
        let target = Some(
            raw.try_take("Target")
                .ok_or(reader.err_missing_parameter(&raw.source, &raw.target.inner, "Target"))?
                .value
                .value()
                .to_string(),
        );

        let path = Some(format_path(
            raw.try_take("Path")
                .ok_or(reader.err_missing_parameter(&raw.source, &raw.target.inner, "Path"))?
                .value
                .value(),
        ));

        Ok(BaseParams {
            target: target.unwrap(),
            path: path.unwrap(),
        })
    }
}

impl Params for AdditionalParams {
    fn unnamed_param() -> Option<&'static str> {
        None
    }

    fn read<B>(_: &LayoutReader, raw: &mut RawBinding<B>) -> Result<Self, XmlLayoutError>
    where
        Self: Sized,
        B: Clone + Debug + PartialEq + Params,
    {
        let mode =
            BindingMode::from_str(&raw.try_take_value("Mode").unwrap_or("ReadOnce".to_string()));
        let fallback = raw.try_take_value("Fallback");
        let converter = raw.try_take_value("Converter");

        Ok(AdditionalParams {
            converter,
            fallback,
            mode,
        })
    }
}
