use bevy::asset::AssetServer;
use bevy::prelude::{BackgroundColor, Color, EntityCommands};
use crate::injector::Injector;
use crate::prelude::{Extractor, FromStrTyped, ValueStorage};
use crate::xml_component::XmlComponent;

#[derive(Default, Debug, Clone)]
pub struct BackgroundColorParser {
    value: Color
}

fn set_value_safe(color: &mut Color, name: &str, value: &ValueStorage) {
    if name == "value" {
        *color = *value.read::<Color>();
    }
}

impl XmlComponent for BackgroundColorParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        set_value_safe(&mut self.value, name, value)
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(BackgroundColor(self.value));
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(ColorInjector)
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        if name == "value" {
            self.value = Color::from_str_typed(value).unwrap();
            return true;
        }

        false
    }
}

struct ColorInjector;
impl Injector for ColorInjector {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<BackgroundColor, _>(|c| set_value_safe(&mut c.0, name, value));
    }
}