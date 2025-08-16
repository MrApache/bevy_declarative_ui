mod utils;

use bevy_declarative_ui_parser::LayoutReader;
use bevy_declarative_ui_parser::errors::XmlLayoutError;
use bevy_declarative_ui_parser::position::Location;
use utils::*;

#[test]
fn layout() {
    let (content, file) = load("unexpected_eof_layout.xml");
    let layout = LayoutReader::new(&content, &file).parse();

    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::EndOfFile {
            file,
            location: Location::new(1, 1, 9),
        }
    );
}

#[test]
fn container() {
    let (content, file) = load("unexpected_eof_container.xml");
    let layout = LayoutReader::new(&content, &file).parse();

    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::EndOfFile {
            file,
            location: Location::new(1, 4, 16),
        }
    );
}

#[test]
fn component() {
    let (content, file) = load("unexpected_eof_component.xml");
    let layout = LayoutReader::new(&content, &file).parse();

    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::EndOfFile {
            file,
            location: Location::new(8, 2, 8),
        }
    );
}
