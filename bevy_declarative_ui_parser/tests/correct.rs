use bevy_declarative_ui_parser::{
    values::{
        AttributeValue,
        TemplateBinding,
        bindings::{
            Binding,
            BindingKind,
            BindingMode,
            params::{
                AdditionalParams,
                BaseParams,
                ItemBaseParams
            },
        },
    },
    Id,
    LayoutReader,
};

mod utils;
use utils::*;

#[test]
fn test() {
    let (content, file) = load("correct.xml");
    let layout = LayoutReader::new(&content, &file).parse();
    if let Err(error) = layout {
        panic!("{}", error);
    }
    let layout = layout.unwrap();

    assert_eq!(layout.root_nodes.len(), 1);

    let container = layout.root_nodes.get(0).unwrap();
    container.has(0, 1, 1, Id::Custom("Root".into()));

    container.children.get(0).unwrap().has(1, 1, 0, Id::Custom("PlayerList".into()));

    let node = container.components.get(0).unwrap();
    node.has("Node", 2);
    node.has_attribute("width", AttributeValue::Value("100%".into()));
    node.has_attribute("height", AttributeValue::Value("100%".into()));

    assert_eq!(layout.templates.len(), 1);
    let template = layout.templates.get(0).unwrap();
    template.has(Id::Template(0), Id::Custom("Root".into()), 1, TemplateBinding::Resource(Binding {
        base_params: BaseParams {
            target: "Players".into(),
            path: "online".into(),
        },
        additional_params: (),
        kind: BindingKind::Resource,
    }));

    let container = template.nodes.get(0).unwrap();
    container.has(0, 1, 0, Id::Template(0));
    let component = container.components.get(0).unwrap();
    component.has("ImageBox", 3);
    component.has_attribute("width", AttributeValue::Resource(Binding {
        base_params: BaseParams {
            target: "Globals".into(),
            path: "width".into(),
        },
        additional_params: AdditionalParams {
            converter: None,
            fallback: None,
            mode: BindingMode::ReadOnce,
        },
        kind: BindingKind::Resource,
    }));

    component.has_attribute("height", AttributeValue::Value("15px".into()));
    component.has_attribute("image", AttributeValue::Item(Binding {
        base_params: ItemBaseParams {
            path: "avatar".into(),
        },
        additional_params: AdditionalParams {
            converter: Some("AsSprite".into()),
            fallback: None,
            mode: BindingMode::Read,
        },
        kind: BindingKind::Item,
    }))
}


