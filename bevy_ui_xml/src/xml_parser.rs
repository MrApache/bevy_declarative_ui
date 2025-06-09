use std::collections::HashMap;
use std::str::Utf8Error;
use quick_xml::events::attributes::Attribute;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use thiserror::Error;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

#[derive(Default)]
pub struct XmlLoader;


#[derive(Clone, Debug, Default, Reflect)]
pub struct Resources {
    properties: HashMap<String, String>
}

#[macro_export]
macro_rules! res {
    ( $( ($key:expr, $value:expr) ),* $(,)? ) => {
        {
            bevy_ui_xml::prelude::Resources::with([ $( ($key, $value) ),* ])
        }
    };
}

impl Resources {
    pub fn insert(&mut self, name: &str, value: impl Into<String>) {
        self.properties.insert(name.to_string(), value.into());
    }

    pub fn with<const N: usize>(pairs:[(&str, &str); N]) -> Self {
        let mut instance = Self { properties: Default::default() };
        for (name, value) in pairs {
            instance.insert(name, value);
        }

        instance
    }

    pub(crate) fn contains_key(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    pub(crate) fn get(&self, name: &str) -> Option<&String> {
        self.properties.get(name)
    }

    pub(crate) fn iter(&self) -> std::collections::hash_map::Iter<String, String> {
        self.properties.iter()
    }
}

#[derive(Default, Debug)]
pub(crate) struct Template {
    pub name: String,
    pub nodes: Vec<UiNode>,
}

#[derive(Default, PartialEq, Debug)]
pub(crate) enum Tag {
    #[default]
    Container,
    Component(String),
}

#[derive(PartialEq, Default, Debug)]
pub(crate) struct NodeAttribute {
    pub name:  String,
    pub value: String,
    pub is_property: bool,
}

#[derive(Default, Debug)]
pub(crate) struct UiNode {
    pub tag:        Tag,
    pub attributes: Vec<NodeAttribute>,
    pub children:   Vec<UiNode>,
}

#[derive(Asset, TypePath, Default, Debug)]
pub struct XmlLayout {
    pub(crate) local:      Resources,
    pub(crate) global:     Resources,
    pub(crate) templates:  Vec<Template>,
    pub(crate) root_nodes: Vec<UiNode>,
}

impl XmlLayout {
    pub(crate) fn get_resource(&self, key: &str) -> Option<&String> {
        if let Some(local_value) = self.local.get(key) {
            Some(local_value)
        }
        else if let Some(global_value) = self.global.get(key) {
            Some(global_value)
        }
        else {
            None
        }
    }
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

    #[error("Missing property name")]
    MissingPropertyName,

    #[error("Empty property name")]
    EmptyPropertyName,

    #[error("Empty template")]
    EmptyTemplate, //Warning

    #[error("Missing template name")]
    MissingTemplateName,

    #[error("Empty template name")]
    EmptyTemplateName,

    #[error("Unexpected tag")]
    UnexpectedTag {
        current: String,
        expected: Vec<&'static str>,
    },

    #[error("Mismatched end tag")]
    MismatchedEndTag {
        current: String,
        expected: &'static str,
    }
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
            (MissingPropertyName, MissingPropertyName) => true,
            (EmptyPropertyName, EmptyPropertyName) => true,
            (EmptyTemplate, EmptyTemplate) => true,
            (MissingTemplateName, MissingTemplateName) => true,
            (EmptyTemplateName, EmptyTemplateName) => true,
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

fn is_property(s: &[u8]) -> bool {
    s.len() > 2 && s.starts_with(b"{") && s.ends_with(b"}")
}

fn extract_property_name(s: &[u8]) -> Result<String, XmlLayoutError> {
    Ok(string(&s[1..s.len() - 1])?)
}

fn parse_attributes(e: BytesStart) -> Result<Vec<NodeAttribute>, XmlLayoutError> {
    e.attributes()
        .map(|res_attr| {
            let attr = res_attr.unwrap();
            let name = string(attr.key.as_ref())?;

            let is_property = is_property(attr.value.as_ref());
            let value = if is_property {
                extract_property_name(attr.value.as_ref())?
            }
            else {
                string(attr.value.as_ref())?
            };

            Ok(NodeAttribute {
                name,
                value,
                is_property,
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

fn parse_property(e: &BytesStart) -> Result<(String, String), XmlLayoutError>{
    let name: Option<Attribute> = e
        .try_get_attribute("name")
        .expect("Missing 'name' attribute_1");

    if name.is_none() {
        return Err(XmlLayoutError::MissingPropertyName);
    }

    let name_attribute: Attribute = name.unwrap();
    let name: &[u8] = name_attribute.value.as_ref();
    if name == b"" {
        return Err(XmlLayoutError::EmptyPropertyName);
    }

    let value = e
        .try_get_attribute("value")
        .expect("Missing 'value' attribute_1");

    let value: String = if let Some(value) = value {
        string(value.value.as_ref())?
    }
    else {
        String::new()
    };

    Ok((string(name)?, value))
}

enum Resource {
    Local,
    Global
}

fn parse_resources(
    reader:    &mut Reader<&[u8]>,
    mut buf:   &mut Vec<u8>,
    resources: &mut Resources,
    resource:  Resource
) -> Result<(), XmlLayoutError>
{
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

                let (key, value) = parse_property(&e)?;
                resources.properties.insert(key, value);
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

fn parse_template(reader: &mut Reader<&[u8]>, evt: Event) -> Result<Template, XmlLayoutError> {
    let mut buf = Vec::new();
    let Event::Start(e) = evt else {
        panic!("Expected Event::Start");
    };

    let name_attribute = e.try_get_attribute("name").expect("template_name_1");
    if name_attribute.is_none() {
        return Err(XmlLayoutError::MissingTemplateName);
    }

    let name = name_attribute.unwrap().value;
    if name.as_ref() == b"" {
        return Err(XmlLayoutError::EmptyTemplateName);
    }

    let mut template: Template = Default::default();
    template.name = string(name.as_ref())?;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                panic!("Error at position {}: {:?}", reader.error_position(), e)
            },
            Ok(Event::Empty(e)) => template.nodes.push(parse_component(Event::Empty(e))?),
            Ok(Event::Start(e)) => {
                if e.name().as_ref() != b"Container" {
                    return Err(XmlLayoutError::UnexpectedTag {
                        current: string(e.name().as_ref())?,
                        expected: vec!["Container"],
                    });
                }
                template.nodes.push(parse_container(reader, Event::Start(e))?);
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

fn parse_layout(str: &str) -> Result<XmlLayout, XmlLayoutError> {
    let mut reader = quick_xml::Reader::from_str(str);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    let mut layout: XmlLayout = Default::default();
    let mut has_layout_tag: bool = false;
    let mut has_global_resources: bool = false;
    let mut has_local_resources: bool = false;

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
                        if !has_layout_tag {
                            has_layout_tag = true;
                            continue;
                        }

                        return Err(XmlLayoutError::MultipleLayouts);
                    }
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
                        parse_resources(&mut reader, &mut buf, &mut layout.global, Resource::Global)?
                    },
                    b"LocalResources" => {
                        if !has_layout_tag {
                            return Err(XmlLayoutError::MissingLayout);
                        }
                        has_local_resources = true;
                        parse_resources(&mut reader, &mut buf, &mut layout.local, Resource::Local)?
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
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Layout" => has_layout_tag = false,
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

impl AssetLoader for XmlLoader {
    type Asset = XmlLayout;
    type Settings = ();
    type Error = XmlLayoutError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _: &(),
        _: &mut LoadContext<'_>,

    ) -> Result<Self::Asset, Self::Error>
    {
        let mut string:String = String::new();
        reader.read_to_string(&mut string).await?;
        parse_layout(&string)
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
    }
}

#[cfg(test)]
mod tests {
    use crate::xml_parser::{parse_layout, NodeAttribute, Tag, XmlLayoutError};

    #[test]
    fn xml_parse() {
        let layout = parse_layout(CORRECT_XML);
        let layout = layout.ok().unwrap();

        assert!(layout.global.properties.contains_key("font"));
        assert_eq!(layout.global.properties.get("font").unwrap(), "fonts/arial.ttf");

        assert!(layout.global.properties.contains_key("font_color"));
        assert_eq!(layout.global.properties.get("font_color").unwrap(), "Red");

        assert!(layout.global.properties.contains_key("font_size"));
        assert_eq!(layout.global.properties.get("font_size").unwrap(), "12");

        assert!(layout.local.properties.contains_key("text"));
        assert_eq!(layout.local.properties.get("text").unwrap(), "Hello world!");

        assert_eq!(layout.templates.get(0).unwrap().name, "button");

        assert_eq!(layout.root_nodes.len(), 2);

        let node = layout.root_nodes.get(0).unwrap();
        assert_eq!(node.tag, Tag::Component("Node".to_string()));
        assert_eq!(node.children.len(), 0);

        assert!(node.attributes.contains(&NodeAttribute {
            name:  "width".to_string(),
            value: "100%".to_string(),
            is_property: false,
        }));

        assert!(node.attributes.contains(&NodeAttribute {
            name:  "height".to_string(),
            value: "100%".to_string(),
            is_property: false,
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
            value: "15px".to_string(),
            is_property: false,
        }));

        assert!(image_box.attributes.contains(&NodeAttribute {
            name:  "height".to_string(),
            value: "15px".to_string(),
            is_property: false,
        }));

        assert!(image_box.attributes.contains(&NodeAttribute {
            name:  "image".to_string(),
            value: "ui/menu.png".to_string(),
            is_property: false,
        }));
    }

    const CORRECT_XML: &str = r#"
    <Layout>
        <GlobalResources>
            <Property name="font" value="fonts/arial.ttf"/>
            <Property name="font_color" value="Red"/>
            <Property name="font_size" value="12"/>
        </GlobalResources>

        <LocalResources>
            <Property name="text" value="Hello world!"/>
        </LocalResources>

        <Template name="button">
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
        assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingPropertyName);
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
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyPropertyName);
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
        assert!(layout.is_ok());
        assert!(layout.unwrap().local.properties.get("Display").unwrap().is_empty());
    }

    const PROPERTY_DEFAULT_VALUE: &str = r#"
    <Layout>
        <LocalResources>
            <Property name="Display"/>
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
        <Template name="button"></Template>
    </Layout>
    "#;

    #[test]
    fn missing_template_name_error() {
        let layout = parse_layout(&MISSING_TEMPLATE_NAME);
        assert_eq!(layout.unwrap_err(), XmlLayoutError::MissingTemplateName);
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
        assert_eq!(layout.unwrap_err(), XmlLayoutError::EmptyTemplateName);
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
            <Property name="value"/>
    "#;

    const UNEXPECTED_EOF_TEMPLATE: &str = r#"
    <Layout>
        <Template name="template">
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
        <Template name="template">
            <GlobalResources>
                <Property name="font_size" value="16"/>
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
            <Property name="font_size" value="16"/>
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