use xml_parser::span::{ErrorSpan, Location};
use crate::error::XmlLayoutError;
use crate::into::{Attribute, NodeValue};
use crate::layout_reader::LayoutReader;
use crate::resources::PropertyValue;

#[test]
fn xml_parse() {
    let layout = LayoutReader::new(CORRECT_XML, "").parse_layout();
    let layout = layout.unwrap();
    assert!(layout.global.properties.contains_key("font"));
    assert_eq!(
        layout.global.properties.get("font").unwrap(),
        &PropertyValue {
            type_: "String".into(),
            value: "fonts/arial.ttf".into(),
        }
    );

    assert!(layout.global.properties.contains_key("font_color"));
    assert_eq!(
        layout.global.properties.get("font_color").unwrap(),
        &PropertyValue {
            type_: "Color".into(),
            value: "Red".into(),
        }
    );

    assert!(layout.global.properties.contains_key("font_size"));
    assert_eq!(
        layout.global.properties.get("font_size").unwrap(),
        &PropertyValue {
            type_: "f32".to_string(),
            value: "12".to_string(),
        }
    );

    assert!(layout.local.properties.contains_key("text"));
    assert_eq!(
        layout.local.properties.get("text").unwrap(),
        &PropertyValue {
            type_: "String".to_string(),
            value: "Hello world!".to_string(),
        }
    );

    assert_eq!(layout.templates.get(0).unwrap().name, "button");

    assert_eq!(layout.root_nodes.len(), 2);

    let node = layout.root_nodes.get(0).unwrap();
    assert_eq!(node.tag.name, "Node");
    assert_eq!(node.children.len(), 0);

    assert!(node.tag.attributes.contains(&Attribute {
        name: "width".to_string(),
        value: NodeValue::Value("100%".into())
    }));

    assert!(node.tag.attributes.contains(&Attribute {
        name: "height".to_string(),
        value: NodeValue::Value("100%".into())
    }));

    let container = layout.root_nodes.get(1).unwrap();
    assert!(container.tag.is_container);
    assert_eq!(container.children.len(), 1);
    assert_eq!(container.tag.attributes.len(), 0);

    let image_box = container.children.get(0).unwrap();
    assert_eq!(image_box.tag.name, "ImageBox");
    assert_eq!(image_box.children.len(), 0);

    assert!(image_box.tag.attributes.contains(&Attribute {
        name: "width".to_string(),
        value: NodeValue::Value("15px".into())
    }));

    assert!(image_box.tag.attributes.contains(&Attribute {
        name: "height".to_string(),
        value: NodeValue::Value("15px".into())
    }));

    assert!(image_box.tag.attributes.contains(&Attribute {
        name: "image".to_string(),
        value: NodeValue::Value("ui/menu.png".into())
    }));
}

const CORRECT_XML: &str = r#"
<Layout>
    <GlobalResources>
        <Property name="font" type="String" value="fonts/arial.ttf"/>
        <Property name="font_color" type="Color" value="Red"/>
        <Property name="font_size" type="f32" value="12"/>
    </GlobalResources>

    <LocalResources>
        <Property name="text" type="String" value="Hello world!"/>
    </LocalResources>

    <Template name="button" container="container">
        <Container on_click="print_message">
            <Node width="10%" height="10%"/>
            <Container>
                <Node width="100px" height="100px"/>
                <BackgroundColor value="Green"/>
                <Button/>
            </Container>
        </Container>
    </Template>

    <Node width="100%" height="100%"/>
    <Container>
        <ImageBox width="15px" height="15px" image="ui/menu.png"/>
    </Container>
</Layout>"#;


#[test]
fn missing_layout_error() {
    const MISSING_LAYOUT_XML: &str = r#"
    <GlobalResources>
        <Property name="font" value="fonts/arial.ttf"/>
    </GlobalResources>
"#;
    let layout = LayoutReader::new(MISSING_LAYOUT_XML, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingLayout {
        file: String::new(),
    });
}

#[test]
fn empty_body_layout_error() {
    const EMPTY_BODY_LAYOUT_XML: &str = r#"<Layout></Layout>"#;
    let layout = LayoutReader::new(EMPTY_BODY_LAYOUT_XML, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLayout);
}

#[test]
fn empty_layout_error() {
    const EMPTY_LAYOUT_XML: &str = r#"<Layout/>"#;
    let layout = LayoutReader::new(EMPTY_LAYOUT_XML, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLayout);
}

#[test]
fn empty_body_global_resources_warning() {
    const EMPTY_BODY_GLOBAL_RESOURCES: &str = r#"
<Layout>
    <GlobalResources></GlobalResources>
</Layout>
"#;
    let layout = LayoutReader::new(EMPTY_BODY_GLOBAL_RESOURCES, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyGlobalResources);
}

#[test]
fn empty_global_resources_warning() {
    const EMPTY_GLOBAL_RESOURCES: &str = r#"
<Layout>
    <GlobalResources/>
</Layout>
"#;
    let layout = LayoutReader::new(EMPTY_GLOBAL_RESOURCES, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyGlobalResources);
}

#[test]
fn empty_body_local_resources_warning() {
    const EMPTY_BODY_LOCAL_RESOURCES: &str = r#"
<Layout>
    <LocalResources></LocalResources>
</Layout>
"#;
    let layout = LayoutReader::new(EMPTY_BODY_LOCAL_RESOURCES, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLocalResources);
}

#[test]
fn empty_local_resources_warning() {
    const EMPTY_LOCAL_RESOURCES: &str = r#"
<Layout>
    <LocalResources/>
</Layout>
"#;
    let layout = LayoutReader::new(EMPTY_LOCAL_RESOURCES, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLocalResources);
}

#[test]
fn missing_property_name_error() {
    const MISSING_PROPERTY_NAME: &str = r#"
<Layout>
    <GlobalResources>
        <Property value="fonts/arial.ttf"/>
    </GlobalResources>
</Layout>
"#;
    let layout = LayoutReader::new(MISSING_PROPERTY_NAME, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::MissingAttribute {
            file: String::new(),
            location: Location::new(31, 4, 10),
            error: ErrorSpan::new("<Property value=\"fonts/arial.ttf\"/>".to_string(), 1, 8),
            attribute: "name"
        }
    );
}

#[test]
fn empty_property_name_error() {
    const EMPTY_PROPERTY_NAME: &str = r#"
<Layout>
    <GlobalResources>
        <Property name="" value="fonts/arial.ttf"/>
    </GlobalResources>
</Layout>
"#;

    let layout = LayoutReader::new(EMPTY_PROPERTY_NAME, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyAttribute {
        file: String::new(),
        location: Location::new(31, 4, 19),
        error: ErrorSpan::new("<Property name=\"\" value=\"fonts/arial.ttf\"/>".to_string(), 10, 7),
        attribute: "name"
    });
}

#[test]
fn property_default_value() {
    const PROPERTY_DEFAULT_VALUE: &str = r#"
<Layout>
    <LocalResources>
        <Property name="Display" type="Display"/>
    </LocalResources>
</Layout>
"#;
    let layout = LayoutReader::new(PROPERTY_DEFAULT_VALUE, "").parse_layout();
    assert!(layout.is_ok(), "{}", layout.unwrap_err().to_string());
    assert!(layout
        .unwrap()
        .local
        .properties
        .get("Display")
        .unwrap()
        .value
        .is_empty());
}

#[test]
fn empty_body_template_warning() {
    const EMPTY_BODY_TEMPLATE: &str = r#"
<Layout>
    <Template name="button" container="container"></Template>
</Layout>
"#;
    let layout = LayoutReader::new(EMPTY_BODY_TEMPLATE, "").parse_layout();
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyTemplate);
}

#[test]
fn missing_template_name_error() {
    const MISSING_TEMPLATE_NAME: &str = r#"
<Layout>
    <Template>
        <Node width="100%"/>
    </Template>
</Layout>
"#;
    let layout = LayoutReader::new(MISSING_TEMPLATE_NAME, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::MissingAttribute {
            file: String::new(),
            location: Location::new(9, 3, 6),
            error: ErrorSpan::new("<Template>".to_string(), 1, 8),
            attribute: "name"
        }
    );
}

#[test]
fn empty_template_name_error() {
    const EMPTY_TEMPLATE_NAME: &str = r#"
<Layout>
    <Template name="" container="some-container">
        <Node width="100%"/>
    </Template>
</Layout>
"#;
    let layout = LayoutReader::new(EMPTY_TEMPLATE_NAME, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyAttribute {
        file: String::new(),
        location: Location::new(9, 3, 15),
        error: ErrorSpan::new("<Template name=\"\" container=\"some-container\">".to_string(), 10, 7),
        attribute: "name",
    });
}

#[test]
fn unexpected_eof_error() {
    const UNEXPECTED_EOF_LAYOUT: &str = r#"<Layout>"#;

    let layout = LayoutReader::new(UNEXPECTED_EOF_LAYOUT, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile {
        file: String::new(),
        location: Location::new(1, 1, 9),
    });

    const UNEXPECTED_EOF_RESOURCES: &str = r#"
<Layout>
    <GlobalResources>
        <Property name="value" type="i32"/>
"#;

    let layout = LayoutReader::new(UNEXPECTED_EOF_RESOURCES, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile {
        file: String::new(),
        location: Location::new(31, 4, 43),
    });

    const UNEXPECTED_EOF_TEMPLATE: &str = r#"
<Layout>
    <Template name="template" container="container">
"#;

    let layout = LayoutReader::new(UNEXPECTED_EOF_TEMPLATE, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile {
        file: String::new(),
        location: Location::new(61, 4, 1),
    });

    const UNEXPECTED_EOF_CONTAINER: &str = r#"
<Layout>
    <Node/>
    <Container>
        <Node/>
"#;

    let layout = LayoutReader::new(UNEXPECTED_EOF_CONTAINER, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile {
        file: String::new(),
        location: Location::new(37, 5, 16),
    });
}

#[test]
fn incorrect_tag_position_error() {
    const INCORRECT_RESOURCE_TAG_POSITION: &str = r#"
<Layout>
    <Template name="template" container="container">
        <GlobalResources>
            <Property name="font_size" type="i32" value="16"/>
        </GlobalResources>
    </Template>
</Layout>
"#;
    let layout = LayoutReader::new(INCORRECT_RESOURCE_TAG_POSITION, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::UnexpectedTag {
            file: String::new(),
            location: Location::new(62, 4, 25),
            error: ErrorSpan::new("<GlobalResources>".to_string(), 1, 15),
            current: "GlobalResources".into(),
            expected: vec!["Container", "LocalResources"],
        }
    );
    const INCORRECT_TEMPLATE_TAG_POSITION: &str = r#"
<Layout>
    <Container>
        <Template name="template" container="some-container">
            <Node/>
        </Template>
    </Container>
</Layout>
"#;
    let layout = LayoutReader::new(INCORRECT_TEMPLATE_TAG_POSITION, "").parse_layout();
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::UnexpectedTag {
            file: String::new(),
            location: Location::new(25, 4, 60),
            error: ErrorSpan::new("<Template name=\"template\" container=\"some-container\">".to_string(), 1, 8),
            current: "Template".into(),
            expected: vec!["Container", "Any component"],
        }
    );
}

#[test]
fn resources_not_property_tag_error() {
    const RESOURCES_NOT_PROPERTY_TAG: &str = r#"
<Layout>
    <GlobalResources>
        <Button/>
        <Property name="font_size" type="u32" value="16"/>
    </GlobalResources>
</Layout>
"#;
    let layout = LayoutReader::new(RESOURCES_NOT_PROPERTY_TAG, "").parse_layout();
    println!("{}", layout.as_ref().err().unwrap());
    assert_eq!(
        layout.unwrap_err(),
        XmlLayoutError::UnexpectedTag {
            file:     String::new(),
            location: Location::new(31, 4, 18),
            error:    ErrorSpan::new("<Button/>".to_string(), 1, 6),
            current:  "Button".to_string(),
            expected: vec!["Property"],
        }
    )
}

/*    #[test]
    fn incorrect_end_tag_error() {
        let layout = parse_layout(&INCORRECT_CONTAINER_END_TAG);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::MismatchedEndTag {
            current: "Template".to_string(),
            expected: "Container",
        })
    }

    const INCORRECT_CONTAINER_END_TAG: &str = r#"
    <Layout>
        <Container>
        </Template>
    </Layout>
    "#;
*/