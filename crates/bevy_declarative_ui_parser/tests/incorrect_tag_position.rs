mod utils;

use bevy_declarative_ui_parser::LayoutReader;
use utils::*;

#[test]
fn test() {
    let (content, file) = load("incorrect_tag_position.xml");
    let layout = LayoutReader::new(&content, &file).parse();
}
