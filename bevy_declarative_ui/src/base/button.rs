use bevy::asset::AssetServer;
use bevy::prelude::EntityCommands;
use crate::injector::Injector;
use crate::prelude::{Extractor, ValueStorage};
use crate::xml_component::XmlComponent;

#[derive(Debug, Clone)]
pub struct ButtonParser;

impl XmlComponent for ButtonParser {
    fn write_value(&mut self, _: &str, _: &ValueStorage) {}

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(bevy::prelude::Button);
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(Self)
    }
}

impl Injector for ButtonParser {
    fn inject_value(
        &self,
        _: &str,
        _: &ValueStorage,
        _: &mut Extractor,
        _: &AssetServer)
    {}
}