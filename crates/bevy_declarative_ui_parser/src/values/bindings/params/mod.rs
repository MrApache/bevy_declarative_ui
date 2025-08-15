use std::fmt::Debug;
use crate::{LayoutReader, XmlLayoutError};
use crate::values::bindings::raw_binding::RawBinding;

mod item_params;
mod component_params;
mod base_params;

pub use item_params::*;
pub use component_params::*;
pub use base_params::*;

pub trait Params {
    fn unnamed_param() -> Option<&'static str>;
    fn read<B>(reader: &LayoutReader, raw: &mut RawBinding<B>) -> Result<Self, XmlLayoutError>
        where
            Self: Sized,
            B: Clone + Debug + PartialEq + Params;
}

impl Params for () {
    fn unnamed_param() -> Option<&'static str> { None }

    fn read<B>(_: &LayoutReader, _: &mut RawBinding<B>) -> Result<Self, XmlLayoutError>
    where
        Self: Sized,
        B: Clone + Debug + PartialEq + Params{ Ok(()) }
}
