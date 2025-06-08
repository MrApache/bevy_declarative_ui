use bevy::asset::AssetServer;
use bevy::prelude::{EntityCommands, Text};
use crate::prelude::Extractor;
use crate::xml_component::XmlComponent;

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
    fn inject_value(&self, name: &str, value: &str, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<Text, _>(|c| {
            set_value(&mut c.0, name, value);
        })
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(Text(self.value.clone()));
    }

    fn clear(&mut self) {
        self.value.clear();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        set_value(&mut self.value, name, value)
    }
}