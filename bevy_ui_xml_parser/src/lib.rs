use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::Utf8Error;
use quick_xml::events::attributes::Attribute;
use thiserror::Error;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use regex::Regex;

#[derive(Default, Debug)]
pub struct XmlLayout {
    pub local:      RawResources,
    pub global:     RawResources,
    pub templates:  Vec<Template>,
    pub root_nodes: Vec<UiNode>,
    pub usings:     HashSet<String>,
}

#[derive(Clone, Debug, Default)]
//Name, Type, Value
pub struct RawResources {
    properties: HashMap<String, PropertyValue>
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropertyValue {
    pub type_: String,
    pub value: String,
}

impl RawResources {
    pub fn insert(&mut self, name: &str, type_: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(name.to_string(), PropertyValue {
            type_: type_.into(),
            value: value.into(),
        });
    }

    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PropertyValue)> {
        self.properties.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

#[derive(Default, Debug)]
pub struct Template {
    pub name: String,
    pub nodes: Vec<UiNode>,
    pub containers: HashSet<String>,
    pub resources: RawResources,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum Tag {
    #[default]
    Container,
    Component(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct NodeAttribute {
    pub name:  String,
    pub value:  NodeValue,
}

impl Default for NodeAttribute {
    fn default() -> Self {
        Self { 
            name: String::default(),
            value: NodeValue::Value(String::default())
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum NodeValue {
    Value(String),
    Property(String),
    CallFunction {
        name: String,
        args: Vec<String>
    },
    CallPropertyFunction {
        name: String,
        args: Vec<String>
    }
}

impl NodeValue {
    pub fn read_value(&self) -> String {
        match self {
            NodeValue::Value(value) => value.clone(),
            _ => panic!("Todo"),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct UiNode {
    pub tag:        Tag,
    pub attributes: Vec<NodeAttribute>,
    pub children:   Vec<UiNode>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum XmlLayoutError {
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse XML: {0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("Unexpected end of file")]
    EndOfFile,

    #[error("Missing <Layout> tag")]
    MissingLayout,

    #[error("Multiple <Layout> tags")]
    MultipleLayouts,

    #[error("Empty layout")]
    EmptyLayout, //Warning

    #[error("Empty global resources")]
    EmptyGlobalResources, //Warning

    #[error("Empty local resources")]
    EmptyLocalResources,  //Warning

    #[error("Empty template")]
    EmptyTemplate, //Warning

    #[error("Missing attribute: {0}")]
    MissingAttribute(&'static str),

    #[error("Empty attribute: {0}")]
    EmptyAttribute(&'static str),

    #[error("Unexpected tag")]
    UnexpectedTag {
        current: String,
        expected: Vec<&'static str>,
    },

    #[error("Mismatched end tag")]
    MismatchedEndTag {
        current: String,
        expected: &'static str,
    },

    #[error("Excepted value")]
    ExceptedValue
}

impl PartialEq for XmlLayoutError {
    fn eq(&self, other: &Self) -> bool {
        use XmlLayoutError::*;
        match (self, other) {
            (EndOfFile, EndOfFile) => true,
            (MissingLayout, MissingLayout) => true,
            (MultipleLayouts, MultipleLayouts) => true,
            (EmptyLayout, EmptyLayout) => true,
            (EmptyGlobalResources, EmptyGlobalResources) => true,
            (EmptyLocalResources, EmptyLocalResources) => true,
            (EmptyTemplate, EmptyTemplate) => true,
            (MissingAttribute(name), MissingAttribute(name1)) => name == name1,
            (EmptyAttribute(name), EmptyAttribute(name1)) => name == name1,
            (UnexpectedTag { current: a, expected: b }, UnexpectedTag { current: x, expected: y }) => {
                a == x && b == y
            },
            (MismatchedEndTag { current: a, expected: b }, MismatchedEndTag { current: x, expected: y }) => {
                a == x && b == y
            },
            // Не сравниваем Io и Utf8Error
            _ => false,
        }
    }
}

fn string(buf: &[u8]) -> Result<String, XmlLayoutError> {
    let result = str::from_utf8(buf);
    if result.is_ok() {
        Ok(result?.to_string())
    }
    else {
        Err(XmlLayoutError::Utf8Error(result.unwrap_err()))
    }
}

fn parse_args(args_str: &str) -> Vec<String> {
    args_str
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse_attribute_value(s: &[u8]) -> Result<NodeValue, XmlLayoutError> {
    let property = Regex::new(r"^\{([A-Za-z_][A-Za-z0-9_]*)\}$").unwrap();
    let property_function = Regex::new(r"^@\{([A-Za-z_][A-Za-z0-9_]*)\}\(([^()]*)\)$").unwrap();
    let function = Regex::new(r"^@([A-Za-z_][A-Za-z0-9_]*)\(([^()]*)\)$").unwrap();

    let input: String = string(s)?;

    // PropertyFunction
    if let Some(caps) = property_function.captures(&input) {
        let property = caps.get(1).unwrap().as_str();
        let args = parse_args(caps.get(2).map_or("", |m| m.as_str()));
        return Ok(NodeValue::CallPropertyFunction { args, name: property.to_string() });
    }

    // Function
    if let Some(caps) = function.captures(&input) {
        let name = caps.get(1).unwrap().as_str();
        let args = parse_args(caps.get(2).map_or("", |m| m.as_str()));
        return Ok(NodeValue::CallFunction { args, name: name.to_string() });
    }

    // Property
    if let Some(caps) = property.captures(&input) {
        return Ok(NodeValue::Property(caps.get(1).unwrap().as_str().to_string()));
    }

    // Default: Value
    Ok(NodeValue::Value(input.to_string()))
}

fn parse_attributes(e: BytesStart) -> Result<Vec<NodeAttribute>, XmlLayoutError> {
    e.attributes()
        .map(|res_attr| {
            let attr = res_attr.unwrap();
            let name = string(attr.key.as_ref())?;
            let value = parse_attribute_value(&attr.value.as_ref())?;

            Ok(NodeAttribute {
                name,
                value,
            })
        })
        .collect()
}

fn parse_container(reader: &mut Reader<&[u8]>, evt: Event) -> Result<UiNode, XmlLayoutError> {
    let mut buf = Vec::new();
    let Event::Start(e) = evt else {
        panic!("Expected Event::Start");
    };

    let mut node: UiNode = Default::default();
    node.tag = Tag::Container;
    node.attributes = parse_attributes(e)?;

    for attribute in node.attributes.iter() {
        if attribute.name == "id" && !matches!(attribute.value, NodeValue::Value(_)) {
            return Err(XmlLayoutError::ExceptedValue);
        }
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                panic!("Error at position {}: {:?}", reader.error_position(), e)
            },
            Ok(Event::Start(e)) => {
                if e.name().as_ref() != b"Container" {
                    return Err(XmlLayoutError::UnexpectedTag {
                        current: string(e.name().as_ref())?,
                        expected: vec!["Container", "Any component"],
                    });
                }
                node.children.push(parse_container(reader, Event::Start(e))?);
            },
            Ok(Event::Empty(e)) => node.children.push(parse_component(Event::Empty(e))?),
            Ok(Event::End(_)) => return Ok(node),
            Ok(Event::Eof) => return Err(XmlLayoutError::EndOfFile),
            _ => {}
        }

        buf.clear();
    }
}

fn parse_component(evt: Event) -> Result<UiNode, XmlLayoutError> {
    let Event::Empty(e) = evt else {
        panic!("Expected Event::Empty");
    };

    let mut node: UiNode = Default::default();
    node.tag = Tag::Component(string(e.name().as_ref())?);
    node.attributes = parse_attributes(e)?;
    Ok(node)
}

fn parse_property(e: &BytesStart) -> Result<(String, String, String), XmlLayoutError>{
    let name: String = parse_required_attribute(&e, "name")?;
    let type_: String = parse_required_attribute(&e, "type")?;

    let value = e
        .try_get_attribute("value")
        .expect("Missing 'value' attribute_1");

    let value: String = if let Some(value) = value {
        string(value.value.as_ref())?
    }
    else {
        String::new()
    };

    Ok((name, type_, value))
}

enum Resource {
    Local,
    Global
}

fn parse_resources(
    reader:    &mut Reader<&[u8]>,
    resources: &mut RawResources,
    resource:  Resource
) -> Result<(), XmlLayoutError>
{
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                panic!("Error at position {}: {:?}", reader.error_position(), e)
            },
            Ok(Event::Empty(e)) =>{
                if e.name().as_ref() != b"Property" {
                    return Err(XmlLayoutError::UnexpectedTag {
                        current: string(e.name().as_ref())?,
                        expected: vec!["Property"],
                    });
                }

                let (key, type_, value) = parse_property(&e)?;
                resources.properties.insert(key, PropertyValue {
                    type_,
                    value,
                });
            }
            Ok(Event::Start(e)) => panic!("{}", string(e.name().as_ref())?),
            Ok(Event::End(e)) => {
                match resource {
                    Resource::Local => {
                        if e.name().as_ref() == b"LocalResources" {
                            return Ok(());
                        }
                    }
                    Resource::Global => {
                        if e.name().as_ref() == b"GlobalResources" {
                            return Ok(());
                        }
                    }
                }
                panic!("{}", string(e.name().as_ref())?)
            },
            Ok(Event::Eof) => return Err(XmlLayoutError::EndOfFile),
            _ => {}
        }

        buf.clear();
    }
}

fn parse_required_attribute(e:&BytesStart, name: &'static str) -> Result<String, XmlLayoutError> {
    let attribute: Option<Attribute> = e.try_get_attribute(name)
        .expect(&format!("missing attribute: {}", name));

    if attribute.is_none() {
        return Err(XmlLayoutError::MissingAttribute(name));
    }

    let attribute: Cow<[u8]> = attribute.unwrap().value;
    if attribute.as_ref() == b"" {
        return Err(XmlLayoutError::EmptyAttribute(name));
    }

    string(attribute.as_ref())
}

fn parse_template(reader: &mut Reader<&[u8]>, evt: Event) -> Result<Template, XmlLayoutError> {
    let mut buf = Vec::new();
    let Event::Start(e) = evt else {
        panic!("Expected Event::Start");
    };

    let name = parse_required_attribute(&e, "name")?;
    let container = parse_required_attribute(&e, "container")?;
    let container = string(container.as_ref())?;
    let list: HashSet<String> = container
        .trim()
        .split(';')
        .map(|s| s.trim().to_string())
        .collect();

    let mut template: Template = Default::default();
    template.name = string(name.as_ref())?;
    template.containers = list;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                panic!("Error at position {}: {:?}", reader.error_position(), e)
            },
            Ok(Event::Empty(e)) => template.nodes.push(parse_component(Event::Empty(e))?),
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Container" => template.nodes.push(parse_container(reader, Event::Start(e))?),
                    b"LocalResources" => parse_resources(reader, &mut template.resources, Resource::Local)?,
                    _ => {
                        return Err(XmlLayoutError::UnexpectedTag {
                            current: string(e.name().as_ref())?,
                            expected: vec!["Container", "LocalResources"],
                        });
                    }
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Container" => {},
                    b"Template" => {
                        if template.nodes.len() == 0 {
                            return Err(XmlLayoutError::EmptyTemplate);
                        }
                        return Ok(template);
                    },
                    _ => panic!()
                }
            }
            Ok(Event::Eof) => return Err(XmlLayoutError::EndOfFile),
            _ => {}
        }

        buf.clear();
    }
}

pub fn parse_layout(str: &str) -> Result<XmlLayout, XmlLayoutError> {
    let mut reader = quick_xml::Reader::from_str(str);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    let mut layout: XmlLayout = Default::default();
    let mut has_layout_tag: bool = false;
    let mut has_global_resources: bool = false;
    let mut has_local_resources: bool = false;

    let mut using: bool = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),

            Ok(Event::Eof) => {
                if !has_layout_tag {
                    break;
                }

                return Err(XmlLayoutError::EndOfFile);
            },

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Layout" => {
                        if has_layout_tag {
                            return Err(XmlLayoutError::MultipleLayouts);
                        }
                        has_layout_tag = true;
                    }
                    b"Use" => using = true,
                    b"Container" => {
                        if !has_layout_tag {
                            return Err(XmlLayoutError::MissingLayout);
                        }

                        layout.root_nodes.push(parse_container(&mut reader, Event::Start(e))?);
                    }
                    b"GlobalResources" => {
                        if !has_layout_tag {
                            return Err(XmlLayoutError::MissingLayout);
                        }
                        has_global_resources = true;
                        parse_resources(&mut reader, &mut layout.global, Resource::Global)?
                    },
                    b"LocalResources" => {
                        if !has_layout_tag {
                            return Err(XmlLayoutError::MissingLayout);
                        }
                        has_local_resources = true;
                        parse_resources(&mut reader, &mut layout.local, Resource::Local)?
                    },
                    b"Template" => {
                        if !has_layout_tag {
                            return Err(XmlLayoutError::MissingLayout);
                        }
                        layout.templates.push(parse_template(&mut reader, Event::Start(e))?)
                    },
                    unknown => panic!("Unknown tag: {}", string(unknown.as_ref())?),
                }
            },
            Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"Layout"          => return Err(XmlLayoutError::EmptyLayout),
                    b"GlobalResources" => return Err(XmlLayoutError::EmptyGlobalResources),
                    b"LocalResources"  => return Err(XmlLayoutError::EmptyLocalResources),
                    b"Container" => panic!("container"),
                    b"Template" => panic!("template"),
                    _ => layout.root_nodes.push(parse_component(Event::Empty(e))?),
                }
            },
            Ok(Event::Text(e)) => {
                if !using {
                    panic!("TODO");
                }

                layout.usings.insert(e.unescape().unwrap().into_owned());
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Layout" => has_layout_tag = false,
                    b"Use" => using = false,
                    _ => {}
                }
            }
            _ => (),
        }
        buf.clear();
    }

    if layout.root_nodes.is_empty()
        && layout.templates.is_empty()
        && !has_local_resources
        && !has_global_resources {
        return Err(XmlLayoutError::EmptyLayout);
    }

    if has_global_resources && layout.global.properties.is_empty() {
        return Err(XmlLayoutError::EmptyGlobalResources);
    }

    if has_local_resources && layout.local.properties.is_empty() {
        return Err(XmlLayoutError::EmptyLocalResources);
    }

    Ok(layout)
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_layout,
        NodeAttribute,
        NodeValue,
        PropertyValue,
        Tag,
        XmlLayoutError
    };

    #[test]
    fn xml_parse() {
        let layout = parse_layout(CORRECT_XML);
        if layout.is_err() {
            panic!("Failed to parse XML: {}", layout.err().unwrap());
        }
        let layout = layout.ok().unwrap();

        assert!(layout.global.properties.contains_key("font"));
        assert_eq!(layout.global.properties.get("font").unwrap(), &PropertyValue {
            type_: "String".into(),
            value: "fonts/arial.ttf".into(),
        });

        assert!(layout.global.properties.contains_key("font_color"));
        assert_eq!(layout.global.properties.get("font_color").unwrap(), &PropertyValue {
            type_: "Color".into(),
            value: "Red".into(),
        });

        assert!(layout.global.properties.contains_key("font_size"));
        assert_eq!(layout.global.properties.get("font_size").unwrap(), &PropertyValue {
            type_: "f32".to_string(),
            value: "12".to_string(),
        } );

        assert!(layout.local.properties.contains_key("text"));
        assert_eq!(layout.local.properties.get("text").unwrap(), &PropertyValue {
            type_: "String".to_string(),
            value: "Hello world!".to_string(),
        });

        assert_eq!(layout.templates.get(0).unwrap().name, "button");

        assert_eq!(layout.root_nodes.len(), 2);

        let node = layout.root_nodes.get(0).unwrap();
        assert_eq!(node.tag, Tag::Component("Node".to_string()));
        assert_eq!(node.children.len(), 0);

        assert!(node.attributes.contains(&NodeAttribute {
            name:  "width".to_string(),
            value: NodeValue::Value("100%".into())
        }));

        assert!(node.attributes.contains(&NodeAttribute {
            name:  "height".to_string(),
            value: NodeValue::Value("100%".into())
        }));

        let container = layout.root_nodes.get(1).unwrap();
        assert_eq!(container.tag, Tag::Container);
        assert_eq!(container.children.len(), 1);
        assert_eq!(container.attributes.len(), 0);

        let image_box = container.children.get(0).unwrap();
        assert_eq!(image_box.tag, Tag::Component("ImageBox".to_string()));
        assert_eq!(image_box.children.len(), 0);

        assert!(image_box.attributes.contains(&NodeAttribute {
            name:  "width".to_string(),
            value: NodeValue::Value("15px".into())
        }));

        assert!(image_box.attributes.contains(&NodeAttribute {
            name:  "height".to_string(),
            value: NodeValue::Value("15px".into())
        }));

        assert!(image_box.attributes.contains(&NodeAttribute {
            name:  "image".to_string(),
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
    </Layout>
"#;

    #[test]
    fn missing_layout_error() {
        let layout = parse_layout(&MISSING_LAYOUT_XML);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingLayout);
    }

    const MISSING_LAYOUT_XML: &str = r#"
        <GlobalResources>
            <Property name="font" value="fonts/arial.ttf"/>
        </GlobalResources>
    "#;

    #[test]
    fn empty_body_layout_error() {
        let layout = parse_layout(&EMPTY_BODY_LAYOUT_XML);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLayout);
    }

    const EMPTY_BODY_LAYOUT_XML: &str = r#"
    <Layout></Layout>
    "#;

    #[test]
    fn empty_layout_error() {
        let layout = parse_layout(&EMPTY_LAYOUT_XML);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLayout);
    }

    const EMPTY_LAYOUT_XML: &str = r#"
    <Layout/>
    "#;

    #[test]
    fn empty_body_global_resources_warning() {
        let layout = parse_layout(&EMPTY_BODY_GLOBAL_RESOURCES);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyGlobalResources);
    }

    const EMPTY_BODY_GLOBAL_RESOURCES: &str = r#"
    <Layout>
        <GlobalResources></GlobalResources>
    </Layout>
    "#;

    #[test]
    fn empty_global_resources_warning() {
        let layout = parse_layout(&EMPTY_GLOBAL_RESOURCES);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyGlobalResources);
    }

    const EMPTY_GLOBAL_RESOURCES: &str = r#"
    <Layout>
        <GlobalResources/>
    </Layout>
    "#;

    #[test]
    fn empty_body_local_resources_warning() {
        let layout = parse_layout(&EMPTY_BODY_LOCAL_RESOURCES);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLocalResources);
    }

    const EMPTY_BODY_LOCAL_RESOURCES: &str = r#"
    <Layout>
        <LocalResources></LocalResources>
    </Layout>
    "#;

    #[test]
    fn empty_local_resources_warning() {
        let layout = parse_layout(&EMPTY_LOCAL_RESOURCES);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyLocalResources);
    }

    const EMPTY_LOCAL_RESOURCES: &str = r#"
    <Layout>
        <LocalResources/>
    </Layout>
    "#;

    #[test]
    fn missing_property_name_error() {
        let layout = parse_layout(&MISSING_PROPERTY_NAME);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingAttribute("name"));
    }

    const MISSING_PROPERTY_NAME: &str = r#"
    <Layout>
        <GlobalResources>
            <Property value="fonts/arial.ttf"/>
        </GlobalResources>
    </Layout>
    "#;

    #[test]
    fn empty_property_name_error() {
        let layout = parse_layout(&EMPTY_PROPERTY_NAME);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyAttribute("name"));
    }

    const EMPTY_PROPERTY_NAME: &str = r#"
    <Layout>
        <GlobalResources>
            <Property name="" value="fonts/arial.ttf"/>
        </GlobalResources>
    </Layout>
    "#;

    #[test]
    fn property_default_value() {
        let layout = parse_layout(&PROPERTY_DEFAULT_VALUE);
        assert!(layout.is_ok(), "{}", layout.unwrap_err().to_string());
        assert!(layout.unwrap().local.properties.get("Display").unwrap().value.is_empty());
    }

    const PROPERTY_DEFAULT_VALUE: &str = r#"
    <Layout>
        <LocalResources>
            <Property name="Display" type="Display"/>
        </LocalResources>
    </Layout>
    "#;

    #[test]
    fn empty_body_template_warning() {
        let layout = parse_layout(&EMPTY_BODY_TEMPLATE);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyTemplate);
    }
    //TODO empty template name

    const EMPTY_BODY_TEMPLATE: &str = r#"
    <Layout>
        <Template name="button" container="container"></Template>
    </Layout>
    "#;

    #[test]
    fn missing_template_name_error() {
        let layout = parse_layout(&MISSING_TEMPLATE_NAME);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingAttribute("name"));
    }

    const MISSING_TEMPLATE_NAME: &str = r#"
    <Layout>
        <Template>
            <Node width="100%"/>
        </Template>
    </Layout>
    "#;

    #[test]
    fn empty_template_name_error() {
        let layout = parse_layout(&EMPTY_TEMPLATE_NAME);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyAttribute("name"));
    }

    const EMPTY_TEMPLATE_NAME: &str = r#"
    <Layout>
        <Template name="">
            <Node width="100%"/>
        </Template>
    </Layout>
    "#;

    #[test]
    fn unexpected_eof_error() {
        let layout = parse_layout(&UNEXPECTED_EOF_LAYOUT);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile);

        let layout = parse_layout(&UNEXPECTED_EOF_RESOURCES);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile);

        let layout = parse_layout(&UNEXPECTED_EOF_TEMPLATE);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile);

        let layout = parse_layout(&UNEXPECTED_EOF_CONTAINER);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EndOfFile);
    }

    const UNEXPECTED_EOF_LAYOUT: &str = r#"
    <Layout>
    "#;

    const UNEXPECTED_EOF_RESOURCES: &str = r#"
    <Layout>
        <GlobalResources>
            <Property name="value" type="i32"/>
    "#;

    const UNEXPECTED_EOF_TEMPLATE: &str = r#"
    <Layout>
        <Template name="template" container="container">
    "#;

    const UNEXPECTED_EOF_CONTAINER: &str = r#"
    <Layout>
        <Node/>
        <Container>
            <Node/>
    "#;

    #[test]
    fn incorrect_tag_position_error() {
        let layout = parse_layout(&INCORRECT_RESOURCE_TAG_POSITION);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::UnexpectedTag {
            current: "GlobalResources".into(),
            expected: vec!["Container"],
        });

        let layout = parse_layout(&INCORRECT_TEMPLATE_TAG_POSITION);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::UnexpectedTag {
            current: "Template".into(),
            expected: vec![
                "Container",
                "Any component"
            ],
        });
    }

    const INCORRECT_RESOURCE_TAG_POSITION: &str = r#"
    <Layout>
        <Template name="template" container="container">
            <GlobalResources>
                <Property name="font_size" type="i32" value="16"/>
            </GlobalResources>
        </Template>
    </Layout>
    "#;

    const INCORRECT_TEMPLATE_TAG_POSITION: &str = r#"
    <Layout>
        <Container>
            <Template name="template">
                <Node/>
            </Template>
        </Container>
    </Layout>
    "#;

    #[test]
    fn resources_not_property_tag_error() {
        let layout = parse_layout(&RESOURCES_NOT_PROPERTY_TAG);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::UnexpectedTag {
            current: "Button".to_string(),
            expected: vec!["Property"],
        })
    }

    const RESOURCES_NOT_PROPERTY_TAG: &str = r#"
    <Layout>
        <GlobalResources>
            <Button/>
            <Property name="font_size" type="u32" value="16"/>
        </GlobalResources>
    </Layout>
    "#;

    //TODO Tests

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
}