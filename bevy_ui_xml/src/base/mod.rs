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

use crate::loader::UiLayoutLoader;

pub fn add_base(library: &mut UiLayoutLoader) {
    library.components.insert("BackgroundColor", || Box::new(BackgroundColorParser::default()));
    library.components.insert("Node",            || Box::new(NodeParser::default()));
    library.components.insert("ImageNode",       || Box::new(ImageNodeParser::default()));
    library.components.insert("TextFont",        || Box::new(TextFontParser::default()));
    library.components.insert("TextColor",       || Box::new(TextColorParser::default()));
    library.components.insert("TextLayout",      || Box::new(TextLayoutParser::default()));
    library.components.insert("Text",            || Box::new(TextParser::default()));
    library.components.insert("Button",          || Box::new(ButtonParser));
}