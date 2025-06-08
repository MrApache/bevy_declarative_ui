use bevy::asset::AssetServer;
use bevy::prelude::{Color, EntityCommands, TextColor};
use crate::prelude::Extractor;
use crate::types::color_str;
use crate::xml_component::XmlComponent;

#[derive(Default, Debug, Clone)]
pub struct TextColorParser {
    value: Color
}

impl XmlComponent for TextColorParser {
    fn inject_value(&self, name: &str, value: &str, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<TextColor, _>(|c| {
            if name == "value" {
                c.0 = color_str(value);
            }
        });
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(TextColor(self.value));
    }

    fn clear(&mut self) {
        self.value = Color::default();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        if name == "value" {
            self.value = color_str(value);
            return true;
        }

        false
    }
}