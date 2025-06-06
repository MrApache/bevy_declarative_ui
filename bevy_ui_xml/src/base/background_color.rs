use bevy::asset::AssetServer;
use bevy::prelude::{BackgroundColor, EntityCommands};
use crate::base::ColorParser;
use crate::xml_component::XmlComponent;

#[derive(Default, Debug, Clone)]
pub struct BackgroundColorParser(ColorParser);

impl XmlComponent for BackgroundColorParser {
    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(BackgroundColor(self.0.color));
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        self.0.parse_attribute(name, value)
    }
}
