use bevy_declarative_ui_parser::{LayoutReader, XmlLayoutError};

#[test]
fn test() {
    const MISSING_LAYOUT_XML: &str = r#"
    <GlobalResources>
        <Property name="font" value="fonts/arial.ttf"/>
    </GlobalResources>
"#;
    let layout = LayoutReader::new(MISSING_LAYOUT_XML, "").parse();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingLayout {
        file: String::new(),
    });
}
