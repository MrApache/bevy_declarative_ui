use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::injector::Injector;
use crate::prelude::{Extractor, FromStrTyped, ValueStorage};
use crate::xml_component::XmlComponent;

#[derive(Default, Clone, Debug)]
pub struct TextLayoutParser {
    text_layout: TextLayout,
}

fn set_value_safe(c: &mut TextLayout, name: &str, value: &ValueStorage) {
    match name {
        "justify"   => c.justify   = *value.read::<JustifyText>(),
        "linebreak" => c.linebreak = *value.read::<LineBreak>(),
        _ => {},
    }
}

impl XmlComponent for TextLayoutParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        set_value_safe(&mut self.text_layout, name, value);
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(self.text_layout.clone());
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(TextLayoutInjector)
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "justify" => self.text_layout.justify = JustifyText::from_str_typed(value).unwrap(),
            "linebreak" => self.text_layout.linebreak = LineBreak::from_str_typed(value).unwrap(),
            _ => return false,
        }

        true
    }
}

struct TextLayoutInjector;
impl Injector for TextLayoutInjector {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<TextLayout, _>(|c| set_value_safe(c, name, value));
    }
}