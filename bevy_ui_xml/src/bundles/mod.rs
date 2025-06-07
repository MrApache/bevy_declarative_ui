
mod text;

pub use text::TextBundleParser;
use crate::XmlLibrary;

pub fn add_bundles(library: &mut XmlLibrary) {
    library.add_component("TextBundle", || Box::new(TextBundleParser::default()));
}