use std::collections::HashMap;
use bevy::prelude::*;
use crate::prelude::*;
use crate::XmlLibrary;
use crate::loader::AttributeProperty;
use crate::xml_parser::{Tag, UiNode};

#[derive(Default, Debug)]
pub struct CompiledLayout {
    pub(crate) root:      CompiledNode,
    pub(crate) local:     Resources,
    pub(crate) global:    Resources,
    pub(crate) templates: HashMap<String, CompiledNode>,
}

#[derive(Default, Debug)]
pub(crate) struct CompiledNode {
    pub(crate) components: Vec<(Box<dyn XmlComponent>, Vec<AttributeProperty>)>,

    pub(crate) containers: Vec<CompiledNode>,
    pub(crate) properties: HashMap<String, String>,

    pub(crate) functions:  HashMap<String, String>,
    pub(crate) id: Option<String>,
}

impl Clone for CompiledNode {
    fn clone(&self) -> Self {
        Self {
            components: self.components
                .iter()
                .map(|(comp, attrs)| (dyn_clone::clone_box(&**comp), attrs.clone()))
                .collect(),

            containers: self.containers.clone(),
            properties: self.properties.clone(),
            functions:  self.functions.clone(),
            id: self.id.clone(),
        }
    }
}

pub(crate) struct LayoutCompiler<'a> {
    library: &'a XmlLibrary,
    layout: &'a XmlLayout,
}

impl<'a> LayoutCompiler<'a> {
    pub fn new(library: &'a XmlLibrary, layout: &'a XmlLayout) -> Self {
        Self { library, layout }
    }

    fn compile_container(&self, node: &UiNode) -> CompiledNode {
        let mut compiled_node: CompiledNode = CompiledNode::default();

        node.children.iter().for_each(|node| {
            self.compile_node(&node, &mut compiled_node);
        });

        node.attributes.iter().for_each(|attr| {
            let resolved_value = if attr.is_property {
                if let Some(value) = self.layout.get_resource(&attr.value) {
                    value.clone()
                }
                else {
                    compiled_node.properties.insert(attr.name.clone(), attr.value.clone());
                    return;
                }
            }
            else {
                attr.value.clone()
            };

            let name: &str = &attr.name;

            if name == "id" {
                compiled_node.id = Some(resolved_value);
            }
            else if self.library.functions.contains_key(name) {
                compiled_node.functions.insert(name.to_string(), resolved_value);
            }
            else {
                error!("[Container] Unknown attribute: {}", name);
            }
        });

        compiled_node
    }

    fn compile_component(&self, node: &UiNode) -> (Box<dyn XmlComponent>, Vec<AttributeProperty>) {
        let Tag::Component(ref name) = node.tag else {
            panic!("expected Tag::Component");
        };

        let mut properties: Vec<AttributeProperty> = Vec::new();
        let mut component: Box<dyn XmlComponent> = self.library.get_component(&name);
        node.attributes.iter().for_each(|attr| {
            if attr.is_property {
                properties.push(AttributeProperty {
                    attribute: attr.name.clone(),
                    property:  attr.value.clone(),
                });

                return;
            }

            if !component.parse_attribute(&attr.name, &attr.value) {
                error!("[{}] Unknown attribute: {}", name, attr.name);
            }
        });

        (component, properties)
    }

    fn compile_node(&self, node: &UiNode, compiled_node: &mut CompiledNode) {
        if node.tag == Tag::Container {
            compiled_node.containers.push(self.compile_container(node))
        }
        else {
            compiled_node.components.push(self.compile_component(node))
        }
    }

    pub fn compile(&self) -> CompiledLayout {
        let mut compiled_layout: CompiledLayout = CompiledLayout::default();
        compiled_layout.global = self.layout.global.clone();
        compiled_layout.local = self.layout.local.clone();

        self.layout.templates.iter().for_each(|template| {
            let mut template_node: CompiledNode = CompiledNode::default();
            template.nodes.iter().for_each(|node| {
                self.compile_node(node, &mut template_node);
            });
            compiled_layout.templates.insert(template.name.clone(), template_node);
        });

        let mut root_node: CompiledNode = CompiledNode::default();

        self.layout.root_nodes.iter().for_each(|node| {
            self.compile_node(node, &mut root_node);
        });

        compiled_layout.root = root_node;
        compiled_layout
    }
}