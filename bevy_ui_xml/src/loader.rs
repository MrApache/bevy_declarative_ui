use std::collections::{HashMap, HashSet};
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::log::error;
use bevy::prelude::{Deref, DerefMut, Resource};
use roxmltree::{Document, NodeType};
use thiserror::Error;
use crate::base::add_base;
use crate::bundles::add_bundles;
use crate::UiLayout;
use crate::xml_component::{XmlComponent, XmlComponentFactory};

#[derive(Debug)]
pub(crate) struct ParsedTree {
    pub(crate) components: Vec<Box<dyn XmlComponent>>,
    pub(crate) containers: Vec<ParsedTree>,
    pub(crate) properties: Vec<AttributeProperty>,
    pub(crate) functions:  Functions,
    pub(crate) id: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Functions {
    pub(crate) on_spawn_fn: Option<String>,
    pub(crate) on_released_fn: Option<String>,
}

impl Clone for ParsedTree {
    fn clone(&self) -> Self {
        Self {
            components: self.components.iter().map(|c| dyn_clone::clone_box(&**c)).collect::<Vec<_>>(),
            containers: self.containers.clone(),
            properties: self.properties.clone(),
            functions:  self.functions.clone(),
            id: self.id.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AttributeProperty {
    pub(crate) attribute: String,
    pub(crate) property: String,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum UiLayoutLoaderError {
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub(crate) struct UiTemplate {
    pub(crate) root: ParsedTree,
}

#[derive(Resource)]
pub struct UiLayoutLoader {
    pub(crate) components: HashMap<&'static str, XmlComponentFactory>,
}

#[derive(Default, Deref, DerefMut)]
struct Resources(HashMap<String, String>);

impl UiLayoutLoader {
    pub fn add_component(&mut self, name: &'static str, factory: XmlComponentFactory) {
        self.components.insert(name, factory);
    }

    fn parse_template(&self, root: roxmltree::Node, resources: &Resources) -> (String, UiTemplate) {
        let name = root.attribute("name").expect("[Ui Layout Template] Template does not have a name").to_owned();

        let mut iter = root.children();
        while let Some(val) = iter.next() {
            if val.node_type() != NodeType::Element {
                continue;
            }

            break;
        }

        (name, UiTemplate {
            root: self.parse_tree(root, resources),
        })
    }

    fn get_component(&self, tag: &str) -> Box<dyn XmlComponent> {
        if !self.components.contains_key(tag) {
            panic!("[Ui layout] Unknown tag: {}", tag)
        }

        self.components.get(tag).unwrap()()
    }

    fn parse_container(&self, node: roxmltree::Node, resources: &Resources) -> ParsedTree {
        let mut container = self.parse_tree(node, resources);
        for attribute in node.attributes() {
            let value = attribute.value();
            if is_property(value) {
                let property = extract_property_name(value).unwrap();
                let value = resources.get(property);
                match attribute.name() {
                    "id"         => container.id = Some(value.unwrap().clone()),
                    "on_spawn"   => container.functions.on_spawn_fn = Some(value.unwrap().clone()),
                    "on_release" => container.functions.on_released_fn = Some(value.unwrap().clone()),
                    _ => error!("[Container] Unknown attribute: {}", attribute.name()),
                }
            }
            else {
                match attribute.name() {
                    "id"         => container.id = Some(value.to_string()),
                    "on_spawn"   => container.functions.on_spawn_fn = Some(value.to_string()),
                    "on_release" => container.functions.on_released_fn = Some(value.to_string()),
                    _ => error!("[Container] Unknown attribute: {}", attribute.name()),
                }
            }
        }

        container
    }

    fn parse_component(&self, node: roxmltree::Node, properties: &mut Vec<AttributeProperty>, resources: &Resources) -> Box<dyn XmlComponent> {
        let name: &str = node.tag_name().name();
        let mut component = self.get_component(name);
        for attribute in node.attributes() {
            if is_property(attribute.value()) {
                let property = extract_property_name(attribute.value())
                    .expect(&format!("[{}] Empty property name", name)).to_string();

                if resources.contains_key(&property) {
                    if !component.parse_attribute(attribute.name(), resources.get(&property).unwrap()) {
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

    fn parse_tree(&self, root: roxmltree::Node, resources: &Resources) -> ParsedTree {
        let mut tree: ParsedTree = ParsedTree {
            components: vec![],
            containers: vec![],
            properties: vec![],
            functions: Default::default(),
            id: None,
        };

        root.children().filter(|n| n.is_element()).for_each(|root_child| {
            let name = root_child.tag_name().name();
            match name {
                "Template" => {},
                "Property" => {},
                "Resources" => {},
                "Container" => tree.containers.push(self.parse_container(root_child, &resources)),
                _ => tree.components.push(self.parse_component(root_child, &mut tree.properties, &resources)),
            }
        });

        tree
    }

    fn parse_resources(&self, root: roxmltree::Node) -> Resources {
        let mut resources: HashMap<String, String> = HashMap::new();

        root.children().filter(|n| n.is_element()).for_each(|child| {
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

    fn parse_layout(&self, doc: Document) -> UiLayout {
        let root: roxmltree::Node = doc.root_element();

        if root.tag_name().name() != "Layout" {
            panic!("[Ui Layout] Invalid layout");
        }

        let mut templates: HashMap<String, UiTemplate> = HashMap::new();
        let mut resources: Resources = Resources::default();

        for children in root.children().filter(|child| child.is_element()) {
            match children.tag_name().name() {
                "Template" => {
                    let (name, template) = self.parse_template(children, &resources);
                    templates.insert(name, template);
                },
                "Resources" => {
                    if !resources.is_empty() {
                        error!("[Ui Layout] Multiple resources tag");
                        continue;
                    }
                    resources = self.parse_resources(children);
                },
                _ => break
            }
        }

        for template in &templates {
            println!("Parsed templates: {:?}", template.0)
        }

        UiLayout {
            root: self.parse_tree(root, &resources),
            templates,
        }
    }
}

impl Default for UiLayoutLoader {
    fn default() -> Self {
        let mut loader = UiLayoutLoader {
            components: HashMap::new(),
        };

        add_base(&mut loader);
        add_bundles(&mut loader);

        loader
    }
}

impl AssetLoader for UiLayoutLoader {
    type Asset = UiLayout;
    type Settings = ();
    type Error = UiLayoutLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,

    ) -> bevy::prelude::Result<Self::Asset, Self::Error>
    {
        let mut string:String = String::new();
        reader.read_to_string(&mut string).await?;

        let doc: Document = Document::parse(&string).unwrap();
        Ok(self.parse_layout(doc))
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
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
