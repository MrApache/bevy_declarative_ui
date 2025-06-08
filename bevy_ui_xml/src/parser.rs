use std::collections::HashMap;
use bevy::prelude::*;
use roxmltree::{Document, NodeType};
use crate::loader::{AttributeProperty, ParsedTree, UiTemplate, XmlAsset};
use crate::prelude::XmlComponent;
use crate::{UiLayout, XmlLibrary};
use crate::commands::ResourceCollection;

#[derive(Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Resources(HashMap<String, String>);

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
        self.0.insert(name.to_string(), value.into());
    }

    pub fn with<const N: usize>(pairs:[(&str, &str); N]) -> Self {
        let mut instance = Self { 0: Default::default() };
        for (name, value) in pairs {
            instance.insert(name, value);
        }

        instance
    }
}

struct ParsingContext<'a> {
    library: &'a XmlLibrary,
    resources: &'a ResourceCollection<'a>,
    root: roxmltree::Node<'a, 'a>,
}

fn parse_template(ctx: &ParsingContext) -> (String, UiTemplate) {
    let name = ctx.root.attribute("name")
        .expect("[Ui Layout Template] Template does not have a name").to_owned();

    let mut iter = ctx.root.children();
    while let Some(val) = iter.next() {
        if val.node_type() != NodeType::Element {
            continue;
        }

        break;
    }

    (name, UiTemplate {
        root: parse_tree(ctx),
    })
}

fn parse_container(ctx: &ParsingContext) -> ParsedTree {
    let mut container = parse_tree(ctx);

    for attribute in ctx.root.attributes() {
        let raw_value = attribute.value();

        let resolved_value = if is_property(raw_value) {
            let property = extract_property_name(raw_value).unwrap();

            if let Some(value) = ctx.resources.get(property) {
                value.clone()
            }
            else {
                container.container_properties.insert(
                    attribute.name().to_string(),
                    property.to_string()
                );
                continue;
            }

        } else {
            raw_value.to_string()
        };

        let name: &str = attribute.name();

        if name == "id" {
            container.id = Some(resolved_value);
        }
        else if ctx.library.functions.contains_key(name) {
            container.functions.insert(name.to_string(), resolved_value);
        }
        else {
            error!("[Container] Unknown attribute: {}", name);
        }
    }

    container
}


fn parse_component(ctx: &ParsingContext) -> (Box<dyn XmlComponent>, Vec<AttributeProperty>) {
    let mut properties: Vec<AttributeProperty> = Vec::new();

    let name: &str = ctx.root.tag_name().name();
    let mut component = ctx.library.get_component(name);
    for attribute in ctx.root.attributes() {
        if is_property(attribute.value()) {
            let property = extract_property_name(attribute.value())
                .expect(&format!("[{}] Empty property name", name))
                .to_string();

            if let Some(value) = ctx.resources.get(&property) {
                if !component.parse_attribute(attribute.name(), value) {
                    error!("[{}] Unknown attribute: {}", name, attribute.name());
                }
            }

            properties.push(AttributeProperty {
                attribute: attribute.name().to_string(),
                property
            });

            continue;
        }

        if !component.parse_attribute(attribute.name(), attribute.value()) {
            error!("[{}] Unknown attribute: {}", name, attribute.name());
        }
    }

    (component, properties)
}

fn parse_tree(ctx: &ParsingContext) -> ParsedTree {
    let mut tree: ParsedTree = ParsedTree {
        components: vec![],
        containers: vec![],
        container_properties: HashMap::new(),
        functions: Default::default(),
        id: None,
    };

    ctx.root.children().filter(|n| n.is_element()).for_each(|root_child| {
        let name = root_child.tag_name().name();
        let ctx = ParsingContext {
            library: ctx.library,
            resources: ctx.resources,
            root: root_child,
        };
        match name {
            "Template" => {},
            "Property" => {},
            "GlobalResources" => {},
            "LocalResources" => {},
            "Container" => tree.containers.push(parse_container(&ctx)),
            _ => tree.components.push(parse_component(&ctx)),
        }
    });

    tree
}

fn parse_resources(ctx: &ParsingContext) -> Resources {
    let mut resources: HashMap<String, String> = HashMap::new();

    ctx.root.children().filter(|n| n.is_element()).for_each(|child| {
        match child.tag_name().name() {
            "Property" => {
                let Some(name) = child.attribute("name") else {
                    error!("[Ui Layout] Property must have a name");
                    return;
                };

                let Some(value) = child.attribute("value") else {
                    error!("[Ui Layout] Property must have a value");
                    return;
                };

                resources.insert(name.to_string(), value.to_string());
            }
            other => error!("[Ui Layout]: unknown resource tag {}", other),
        }
    });

    Resources(resources)
}

pub(crate) fn parse_layout(components: &XmlLibrary, xml: &XmlAsset) -> UiLayout {
    let document: Document = Document::parse(&xml.string).unwrap(); //TODO panic when parsing broken xml
    let root: roxmltree::Node = document.root_element();

    if root.tag_name().name() != "Layout" {
        panic!("[Ui Layout] Invalid layout");
    }

    let mut templates: HashMap<String, UiTemplate> = HashMap::new();
    let mut global_res: Resources = Resources::default();
    let mut local_res: Resources = Resources::default();

    for children in root.children().filter(|child| child.is_element()) {
        let resources: ResourceCollection = ResourceCollection::new(&global_res, &local_res);
        let ctx = ParsingContext {
            library: components,
            resources: &resources,
            root: children,
        };
        match children.tag_name().name() {
            "Template" => {
                let (name, template) = parse_template(&ctx);
                templates.insert(name, template);
            },
            "GlobalResources" => {
                if !global_res.is_empty() {
                    error!("[Ui Layout] Multiple resources tag");
                    continue;
                }
                global_res = parse_resources(&ctx);
            },
            "LocalResources" => {
                if !local_res.is_empty() {
                    error!("[Ui Layout] Multiple resources tag");
                    continue;
                }
                local_res = parse_resources(&ctx);
            }
            _ => break
        }
    }

    for template in &templates {
        println!("Parsed templates: {:?}", template.0)
    }

    let resources: ResourceCollection = ResourceCollection::new(&global_res, &local_res);
    let ctx = ParsingContext {
        library: components,
        resources: &resources,
        root,
    };

    UiLayout {
        root: parse_tree(&ctx),
        templates,
        global: global_res,
        local: local_res,
    }
}

fn is_property(s: &str) -> bool {
    s.starts_with('{') && s.ends_with('}') && s.len() > 2
}

fn extract_property_name(s: &str) -> Option<&str> {
    if is_property(s) {
        Some(&s[1..s.len() - 1])
    } else {
        None
    }
}