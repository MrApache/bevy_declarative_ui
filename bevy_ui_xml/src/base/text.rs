use bevy::asset::AssetServer;
use bevy::prelude::{EntityCommands, Text};
use crate::injector::Injector;
use crate::prelude::{Extractor, ValueStorage};
use crate::xml_component::XmlComponent;

pub struct TextInjector;
impl Injector for TextInjector {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<Text, _>(|c| { set_value(&mut c.0, name, value.read::<String>()); })
    }
}

#[derive(Default, Debug, Clone)]
pub struct TextParser {
    value: String,
}

fn set_value(string: &mut String, name: &str, value: &str) -> bool {
    if name == "text" {
        string.clear();
        string.push_str(value);
        return true;
    }

    false
}

impl XmlComponent for TextParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        set_value(&mut self.value, name, value.read::<String>());
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(Text(self.value.clone()));
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(TextInjector)
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        set_value(&mut self.value, name, value)
    }
}