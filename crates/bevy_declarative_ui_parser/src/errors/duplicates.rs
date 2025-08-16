use crate::position::{Location, SimpleErrorSpan};

#[derive(PartialEq, Debug)]
pub struct Duplicates {
    pub file: String,
    pub location: Location,
    pub source: String,
    pub errors: Vec<SimpleErrorSpan>,
}

impl Duplicates {
    pub const fn new(
        file: String,
        location: Location,
        source: String,
        errors: Vec<SimpleErrorSpan>,
    ) -> Self {
        Self {
            file,
            location,
            source,
            errors,
        }
    }
}
