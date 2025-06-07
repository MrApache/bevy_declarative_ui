mod text_color;
mod node;
mod color;
mod background_color;
mod text_font;
mod text_layout;
mod text;
mod image;
mod button;

pub use text_color::TextColorParser;
pub use node::NodeParser;
pub use color::ColorParser;
pub use background_color::BackgroundColorParser;
pub use text_font::TextFontParser;
pub use text_layout::TextLayoutParser;
pub use text::TextParser;
pub use image::ImageNodeParser;
pub use button::ButtonParser;

use crate::XmlLibrary;

pub fn add_base(library: &mut XmlLibrary) {
    library.add_component("BackgroundColor", || Box::new(BackgroundColorParser::default()));
    library.add_component("Node",            || Box::new(NodeParser::default()));
    library.add_component("ImageNode",       || Box::new(ImageNodeParser::default()));
    library.add_component("TextFont",        || Box::new(TextFontParser::default()));
    library.add_component("TextColor",       || Box::new(TextColorParser::default()));
    library.add_component("TextLayout",      || Box::new(TextLayoutParser::default()));
    library.add_component("Text",            || Box::new(TextParser::default()));
    library.add_component("Button",          || Box::new(ButtonParser));
}