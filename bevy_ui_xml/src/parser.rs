use std::collections::HashMap;
use bevy::prelude::*;
use roxmltree::{Document, NodeType};
use crate::loader::{AttributeProperty, ParsedTree, UiTemplate, XmlAsset};
use crate::prelude::XmlComponent;
use crate::{UiLayout, XmlLibrary};

#[derive(Default, Deref, DerefMut)]
struct Resources(HashMap<String, String>);

struct ParsingContext<'a> {
    components: &'a XmlLibrary,
    resources: &'a Resources,
    root: roxmltree::Node<'a, 'a>,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub(crate) struct Layouts(HashMap<AssetId<XmlAsset>, UiLayout>);

pub(crate) fn parse_xml(
    mut events:  EventReader<AssetEvent<XmlAsset>>,
    mut layouts: ResMut<Layouts>,
    components:  Res<XmlLibrary>,
    assets:      Res<Assets<XmlAsset>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id: handle } = event {
            let asset = assets.get(*handle).unwrap();
            layouts.insert(*handle, parse_layout(&components, asset));
        }
    }
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
        let value = attribute.value();
        if is_property(value) {
            let property = extract_property_name(value).unwrap();
            let value = ctx.resources.get(property);
            match attribute.name() {
                "id"       => container.id = Some(value.unwrap().clone()),
                "on_spawn" => container.functions.on_spawn = Some(value.unwrap().clone()),
                "on_click" => container.functions.on_click = Some(value.unwrap().clone()),
                _ => error!("[Container] Unknown attribute: {}", attribute.name()),
            }
        }
        else {
            match attribute.name() {
                "id"       => container.id = Some(value.to_string()),
                "on_spawn" => container.functions.on_spawn = Some(value.to_string()),
                "on_click" => container.functions.on_click = Some(value.to_string()),
                _ => error!("[Container] Unknown attribute: {}", attribute.name()),
            }
        }
    }

    container
}

fn parse_component(ctx: &ParsingContext, properties: &mut Vec<AttributeProperty>) -> Box<dyn XmlComponent> {
    let name: &str = ctx.root.tag_name().name();
    let mut component = ctx.components.get_component(name);
    for attribute in ctx.root.attributes() {
        if is_property(attribute.value()) {
            let property = extract_property_name(attribute.value())
                .expect(&format!("[{}] Empty property name", name))
                .to_string();

            if ctx.resources.contains_key(&property) {
                if !component.parse_attribute(attribute.name(), ctx.resources.get(&property).unwrap()) {
                    error!("[{}] Unknown attribute: {}", name, attribute.name());
                }
            }
            else {
                properties.push(AttributeProperty {
                    attribute: attribute.name().to_string(),
                    property
                });
            }
            continue;
        }

        if !component.parse_attribute(attribute.name(), attribute.value()) {
            error!("[{}] Unknown attribute: {}", name, attribute.name());
        }
    }

    component
}

fn parse_tree(ctx: &ParsingContext) -> ParsedTree {
    let mut tree: ParsedTree = ParsedTree {
        components: vec![],
        containers: vec![],
        properties: vec![],
        functions: Default::default(),
        id: None,
    };

    ctx.root.children().filter(|n| n.is_element()).for_each(|root_child| {
        let name = root_child.tag_name().name();
        let ctx = ParsingContext {
            components: ctx.components,
            resources: ctx.resources,
            root: root_child,
        };
        match name {
            "Template" => {},
            "Property" => {},
            "Resources" => {},
            "Container" => tree.containers.push(parse_container(&ctx)),
            _ => tree.components.push(parse_component(&ctx, &mut tree.properties)),
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

fn parse_layout(components: &XmlLibrary, xml: &XmlAsset) -> UiLayout {
    let document: Document = Document::parse(&xml.string).unwrap();
    let root: roxmltree::Node = document.root_element();

    if root.tag_name().name() != "Layout" {
        panic!("[Ui Layout] Invalid layout");
    }

    let mut templates: HashMap<String, UiTemplate> = HashMap::new();
    let mut resources: Resources = Resources::default();

    for children in root.children().filter(|child| child.is_element()) {
        let ctx = ParsingContext {
            components,
            resources: &resources,
            root: children,
        };
        match children.tag_name().name() {
            "Template" => {
                let (name, template) = parse_template(&ctx);
                templates.insert(name, template);
            },
            "Resources" => {
                if !resources.is_empty() {
                    error!("[Ui Layout] Multiple resources tag");
                    continue;
                }
                resources = parse_resources(&ctx);
            },
            _ => break
        }
    }

    for template in &templates {
        println!("Parsed templates: {:?}", template.0)
    }

    let ctx = ParsingContext {
        components,
        resources: &resources,
        root,
    };

    UiLayout {
        root: parse_tree(&ctx),
        templates,
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