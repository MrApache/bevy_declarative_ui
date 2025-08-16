use bevy_declarative_ui_parser::attribute::Attribute;
use bevy_declarative_ui_parser::into::Tag;
use bevy_declarative_ui_parser::values::{AttributeValue, TemplateBinding};
use bevy_declarative_ui_parser::{Id, ItemTemplate, UiNode};

pub fn load(file: &str) -> (String, String) {
    let work_dir = std::env::current_dir().unwrap().display().to_string();
    let file = format!("{work_dir}/tests/assets/{file}");
    let content = std::fs::read_to_string(&file).unwrap();
    (content, file)
}

pub trait ContainerAssert {
    fn has(&self, attribute_len: usize, component_len: usize, container_len: usize, id: Id);
}

impl ContainerAssert for UiNode {
    fn has(&self, attribute_len: usize, component_len: usize, container_len: usize, id: Id) {
        assert_eq!(self.tag.name, "Container");
        assert_eq!(self.tag.attributes.len(), attribute_len);
        assert_eq!(self.components.len(), component_len);
        assert_eq!(self.children.len(), container_len);
        assert_eq!(self.id, id);
    }
}

pub trait ComponentAssert {
    fn has(&self, name: &'static str, attribute_len: usize);
    fn has_attribute(&self, name: &'static str, value: AttributeValue);
}

impl ComponentAssert for Tag {
    fn has(&self, name: &'static str, attribute_len: usize) {
        assert_eq!(self.name, name);
        assert_eq!(self.attributes.len(), attribute_len);
    }

    fn has_attribute(&self, name: &'static str, value: AttributeValue) {
        assert!(self.attributes.contains(&Attribute {
            name: name.to_string(),
            value,
        }));
    }
}

pub trait TemplateAssert {
    fn has(&self, id: Id, owner: Id, nodes_len: usize, source: TemplateBinding);
}

impl TemplateAssert for ItemTemplate {
    fn has(&self, id: Id, owner: Id, nodes_len: usize, source: TemplateBinding) {
        assert_eq!(self.id, id);
        assert_eq!(self.owner, owner);
        assert_eq!(self.source, source);
        assert_eq!(self.nodes.len(), nodes_len);
    }
}
