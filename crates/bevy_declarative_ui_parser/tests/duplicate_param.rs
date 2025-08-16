mod utils;

use bevy_declarative_ui_parser::LayoutReader;
use bevy_declarative_ui_parser::errors::{Duplicates, XmlLayoutError};
use bevy_declarative_ui_parser::position::{Location, SimpleErrorSpan};
use utils::*;

#[test]
fn single_line() {
    let (content, file) = load("single_line_duplicate_param.xml");
    let result = LayoutReader::new(&content, &file).parse();
    assert_eq!(result.err().unwrap(), XmlLayoutError::DuplicateParam {
        context: Duplicates::new(
            file,
            Location::new(8, 2, 18),
            "{Component Enemy, Target=PlayerEnemy, Target=Camera, Mode=Read, Fallback=100px, Converter=Round, Target=Target}".into(),
            vec![
                SimpleErrorSpan::new(18, 6),
                SimpleErrorSpan::new(38, 6),
                SimpleErrorSpan::new(97, 6),
                SimpleErrorSpan::new(11, 5),
            ]
        ),
        name: "Target".to_string(),
    });
}

#[test]
fn multi_line() {
    let (content, file) = load("multi_line_duplicate_param.xml");
    let result = LayoutReader::new(&content, &file).parse();

    println!("{}", result.err().unwrap());
}
