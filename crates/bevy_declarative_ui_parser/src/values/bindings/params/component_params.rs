use crate::values::bindings::filter::Filters;
use crate::values::bindings::params::Params;
use crate::values::bindings::raw_binding::RawBinding;
use crate::{LayoutReader, XmlLayoutError};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentParams {
    pub filters: Filters,
}

impl Params for ComponentParams {
    fn unnamed_param() -> Option<&'static str> {
        None
    }

    fn read<B>(_: &LayoutReader, raw: &mut RawBinding<B>) -> Result<Self, XmlLayoutError>
    where
        Self: Sized,
        B: Clone + Debug + PartialEq + Params,
    {
        let filters = Filters::from(raw.try_take_value("Fallback").unwrap_or_default().as_str());
        Ok(Self { filters })
    }
}
