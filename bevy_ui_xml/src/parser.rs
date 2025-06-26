use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use bevy::prelude::*;
use bevy_ui_xml_parser::{Resources, NodeValue, UiNode};
use crate::prelude::*;
use crate::resources::Storage;
use crate::xml_parser::LayoutPath;
use crate::XmlLibrary;

#[derive(Default)]
pub struct CompiledLayout {
    pub(crate) root:      CompiledNode,
    pub(crate) local:     UiResources,
    pub(crate) global:    UiResources,
    pub(crate) types:     HashMap<String, TypeId>,
    pub(crate) templates: HashMap<String, Template>,
}

pub(crate) struct Template {
    pub allowed_containers: HashSet<String>,
    //pub local: UiResources,
    pub root: CompiledNode
}

impl Debug for CompiledLayout {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Component {
    pub value: Box<dyn XmlComponent>,
    pub properties: Vec<AttributeProperty>
}

impl Clone for Component {
    fn clone(&self) -> Self {
        Self {
            value: dyn_clone::clone_box(&*self.value),
            properties: self.properties.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub value: String,
    pub kind:  FunctionType
}

#[derive(Debug, Clone)]
pub(crate) enum FunctionType {
    Value,
    CallFunction(String),
}

#[derive(Default, Debug)]
pub(crate) struct CompiledNode {
    pub(crate) components: Vec<Component>,

    pub(crate) containers: Vec<CompiledNode>,
    pub(crate) properties: HashMap<String, String>,

    /// Name -> Function
    pub(crate) functions:  HashMap<String, Function>,
    pub(crate) id: Option<String>,
}

impl Clone for CompiledNode {
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
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

        node.tag.attributes.iter().for_each(|attr| {
            match attr.name.as_str() {
                "id" => compiled_node.id = Some(attr.value.read_value()),
                val if self.library.functions.contains_key(val) => {
                    let function = match &attr.value {
                        NodeValue::Binding(value) => Function {
                            value: value.clone(),
                            kind: FunctionType::Value,
                        },
                        NodeValue::CallFunction { name, args } => {
                            Function {
                                value: name.clone(),
                                kind: FunctionType::CallFunction(args.join("")),
                            }
                        },
                        _ => panic!("Unsupported binding type"),
                    };
                    compiled_node.functions.insert(attr.name.to_string(), function);
                },
                _ => error!("[Container] Unknown attribute: {}", attr.name),
            }
        });

        compiled_node
    }

    fn compile_component(&self, node: &UiNode) -> Component {
        let name = &node.tag.name;
        let mut properties: Vec<AttributeProperty> = Vec::new();
        let mut component: Box<dyn XmlComponent> = self.library.get_component(name);
        node.tag.attributes.iter().for_each(|attr| {

            let value = match &attr.value {
                NodeValue::Value(value) => &value,
                NodeValue::Local(value) | NodeValue::Global(value) => {
                    properties.push(AttributeProperty {
                        attribute: attr.name.clone(),
                        property:  value.clone(),
                    });

                    if let Some(prop) = self.layout.local.get(&value) {
                        &prop.value
                    }
                    else if let Some(prop) = self.layout.global.get(&value) {
                        &prop.value
                    }
                    else {
                        dbg!(&self.layout.local);
                        dbg!(&self.layout.global);
                        panic!("TODO: {}", &value);
                    }
                }
                other => panic!("Unsupported binding type: {other:?}"),
            };

            if !component.parse_attribute(&attr.name, value) {
                error!("[{}] Unknown attribute: {}", name, attr.name);
            }
        });

        Component {
            value: component,
            properties,
        }
    }

    fn compile_node(&self, node: &UiNode, compiled_node: &mut CompiledNode) {
        if node.tag.is_container {
            compiled_node.containers.push(self.compile_container(node))
        }
        else {
            compiled_node.components.push(self.compile_component(node))
        }
    }

    fn get_types(&self) -> HashMap<String, TypeId> {
        let mut map: HashMap<String, TypeId> = HashMap::new();
        self.layout.local.iter().for_each(|(n, _)| {
            let (type_id, _) = self.library.storages.get(self.layout.path.current.as_str())
                .expect(&format!("Path '{}' not found", self.layout.path.current))
                .get(n)
                .expect(&format!("Type '{}' not found on layout: {}", n, self.layout.path.current));
            map.insert(n.to_string(), *type_id);
        });

        self.layout.global.iter().for_each(|(n, _)| {
            let (type_id, _) = self.library.storages.get(self.layout.path.global.as_str())
                .expect(&format!("Path '{}' not found", self.layout.path.global))
                .get(n)
                .expect(&format!("Type '{}' not found on layout: {}", n, self.layout.path.global));
            map.insert(n.to_string(), *type_id);
        });

        map
    }

    fn compile_resources(&self, resources: &mut UiResources, path: &str, raw_resources: &Resources, is_inherit: bool) {
        raw_resources.iter().for_each(|(k, v)| {
            let (id, factory) = self.library.storages.get(path)
                .expect(&format!("Path '{}' not found", path))
                .get(k.as_str()).unwrap();
            let mut storage: Storage = Storage::new(factory(), is_inherit);
            if !is_inherit {
                let parser: &Box<dyn IsTyped> = self.library.types.get(v.type_.as_str())
                    .expect(&format!("Type '{}' on path '{}' not found", v.type_, path));
                parser.write_to_storage(v.value.as_str(), &mut MutValueStorage::new(&mut storage.storage));
            }
            resources.add_property_internal(*id, storage);
        });
    }

    fn compile_local_resources(&self, local: &Resources, global: &Resources) -> UiResources {
        let mut resources: UiResources = UiResources::default();
        self.compile_resources(&mut resources, &self.layout.path.global, global, true);
        self.compile_resources(&mut resources, &self.layout.path.current, local, false);
        resources
    }

    fn compile_global_resources(&self, global: &Resources) -> UiResources {
        let mut resources: UiResources = UiResources::default();
        self.compile_resources(&mut resources, &self.layout.path.global, global, false);
        resources
    }

    pub fn compile(&self) -> CompiledLayout {
        let mut compiled_layout: CompiledLayout = CompiledLayout::default();
        compiled_layout.types = self.get_types();
        compiled_layout.global = self.compile_global_resources(&self.layout.global);
        compiled_layout.local = self.compile_local_resources(&self.layout.local, &self.layout.global);

        self.layout.templates.iter().for_each(|template| {
            let mut layout = XmlLayout {
                path: LayoutPath {
                    current: format!("{}::{}", self.layout.path.current, template.name),
                    global: self.layout.path.global.clone(),
                },
                local: template.resources.clone(),
                root_nodes: template.nodes.clone(),
                global: self.layout.global.clone(),
                templates: vec![],
            };
            let compiled_template: CompiledLayout = LayoutCompiler::new(self.library, &mut layout).compile();
            compiled_layout.types.extend(compiled_template.types);
            compiled_layout.templates.insert(template.name.clone(), Template {
                allowed_containers: template.containers.clone(),
                //local: compiled_template.local,
                root: compiled_template.root,
            });
        });

        let mut root_node: CompiledNode = CompiledNode::default();

        self.layout.root_nodes.iter().for_each(|node| {
            self.compile_node(node, &mut root_node);
        });

        compiled_layout.root = root_node;
        compiled_layout
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Color;
    use bevy_ui_xml_parser::LayoutReader;
    use crate::parser::{CompiledLayout, LayoutCompiler};
    use crate::prelude::{TypedStorage, XmlLayout};
    use crate::resources::PropertyType;
    use crate::xml_parser::LayoutPath;
    use crate::XmlLibrary;

    #[derive(Default)]
    struct Font;
    impl PropertyType for Font {
        type Type = String;
    }

    #[derive(Default)]
    struct FontColor;
    impl PropertyType for FontColor {
        type Type = Color;
    }

    #[derive(Default)]
    struct FontSize;
    impl PropertyType for FontSize {
        type Type = f32;
    }

    #[derive(Default)]
    struct Text;
    impl PropertyType for Text {
        type Type = String;
    }

    const CORRECT_XML: &str = r#"
    <Layout>
        <GlobalResources>
            <Property name="Font" type="String" value="fonts/arial.ttf"/>
            <Property name="FontColor" type="Color" value="Red"/>
            <Property name="FontSize" type="f32" value="12"/>
        </GlobalResources>

        <LocalResources>
            <Property name="Text" type="String" value="Hello, world!"/>
        </LocalResources>

        <Template name="button" container="button_container_small; button_container_big">
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
        <Container id="button_container_small">
            <Node width="50%" height="50%"/>
        </Container>
        <Container id="button_container_big">
            <Node width="50%" height="50%"/>
        </Container>
    </Layout>
"#;
    #[test]
    fn compile_layout() {
        let layout = LayoutReader::new(&CORRECT_XML, "").parse_layout().unwrap();
        let mut layout = XmlLayout {
            path: LayoutPath {
                current: "layout".to_string(),
                global: "layout".to_string(),
            },
            local: layout.local,
            global: layout.global,
            templates: layout.templates,
            root_nodes: layout.root_nodes,
        };
        let mut library = XmlLibrary::default();
        library.add_property::<Font>("layout", "Font", || Box::<TypedStorage<String>>::new(TypedStorage::default()));
        library.add_property::<FontColor>("layout", "FontColor", || Box::<TypedStorage<Color>>::new(TypedStorage::default()));
        library.add_property::<FontSize>("layout", "FontSize", || Box::<TypedStorage<f32>>::new(TypedStorage::default()));
        library.add_property::<Text>("layout", "Text", || Box::<TypedStorage<String>>::new(TypedStorage::default()));
        let compiled_layout: CompiledLayout = LayoutCompiler::new(&library, &mut layout).compile();

        assert_eq!(compiled_layout.global.read_property::<Font>(), "fonts/arial.ttf");
        assert_eq!(compiled_layout.global.read_property::<FontSize>(), &12.0);
        assert_eq!(compiled_layout.global.read_property::<FontColor>(), &Color::srgb(1.0, 0.0, 0.0));

        assert_eq!(compiled_layout.local.read_property::<Text>(), &"Hello, world!");
    }
}