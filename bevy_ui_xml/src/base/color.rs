use bevy::asset::AssetServer;
use bevy::prelude::{Color, EntityCommands};
use crate::xml_component::XmlComponent;
use crate::types::color_str;

#[derive(Default, Debug, Clone)]
pub struct ColorParser {
    pub color: Color
}

impl XmlComponent for ColorParser {
    fn insert_to(&self, _: &mut EntityCommands, _: &AssetServer) {
    }

    fn clear(&mut self) {
        self.color = Color::default();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        if name == "value" {
            self.color = color_str(value);
            return true;
        }

        false
    }
}