
mod text;

pub use text::TextBundleParser;
use crate::loader::UiLayoutLoader;

pub fn add_bundles(library: &mut UiLayoutLoader) {
    library.components.insert("TextBundle", || Box::new(TextBundleParser::default()));
}
