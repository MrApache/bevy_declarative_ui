use crate::values::bindings::format_path;
use crate::values::bindings::params::Params;
use crate::values::bindings::raw_binding::RawBinding;
use crate::{LayoutReader, XmlLayoutError};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub struct ItemBaseParams {
    pub path: String,
}

impl Params for ItemBaseParams {
    fn unnamed_param() -> Option<&'static str> {
        Some("Path")
    }

    fn read<B>(reader: &LayoutReader, raw: &mut RawBinding<B>) -> Result<Self, XmlLayoutError>
    where
        Self: Sized,
        B: Clone + Debug + PartialEq + Params,
    {
        let path = Some(format_path(
            raw.try_take("Path")
                .ok_or(reader.err_missing_parameter(&raw.source, &raw.target.inner, "Path"))?
                .value
                .value(),
        ));

        Ok(ItemBaseParams {
            path: path.unwrap().to_string(),
        })
    }
}
