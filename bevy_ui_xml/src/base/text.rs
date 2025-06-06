use bevy::asset::AssetServer;
use bevy::prelude::{EntityCommands, Text};
use crate::xml_component::XmlComponent;

#[derive(Default, Debug, Clone)]
pub struct TextParser {
    value: String,
}

impl XmlComponent for TextParser {
    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(Text(self.value.clone()));
    }

    fn clear(&mut self) {
        self.value.clear();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        if name == "text" {
            self.value = value.to_string();
            return true;
        }

        false
    }
}