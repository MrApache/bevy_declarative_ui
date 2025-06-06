use bevy::asset::AssetServer;
use bevy::prelude::EntityCommands;
use crate::xml_component::XmlComponent;

#[derive(Debug, Clone)]
pub struct ButtonParser;

impl XmlComponent for ButtonParser {
    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(bevy::prelude::Button);
    }

    fn clear(&mut self) {
    }

    fn parse_attribute(&mut self, _: &str, _: &str) -> bool {
        false
    }
}