use std::str::FromStr;
use bevy::asset::AssetServer;
use bevy::prelude::{EntityCommands, Font, TextFont};
use bevy::text::{FontSmoothing, LineHeight};
use crate::injector::Injector;
use crate::prelude::{Extractor, FromStrTyped, ValueStorage};
use crate::xml_component::XmlComponent;
use crate::raw_handle::RawHandle;

pub struct TextFontInjector;
impl Injector for TextFontInjector {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, server: &AssetServer) {
        extractor.extract::<TextFont, _>(|c| {
            match name {
                "font"           => c.font           = value.load::<Font>(server),
                "font_size"      => c.font_size      = *value.read::<f32>(),
                "line_height"    => c.line_height    = *value.read::<LineHeight>(),
                "font_smoothing" => c.font_smoothing = *value.read::<FontSmoothing>(),
                _ => {}
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct TextFontParser {
    font:  RawHandle<Font>,
    font_size: f32,
    line_height: LineHeight,
    font_smoothing: FontSmoothing
}

impl Default for TextFontParser {
    fn default() -> Self {
        Self {
            font: Default::default(),
            font_size: 20.0,
            line_height: Default::default(),
            font_smoothing: Default::default(),
        }
    }
}

impl XmlComponent for TextFontParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        match name {
            "font"           => self.font           = RawHandle::new(value.read::<String>().clone()),
            "font_size"      => self.font_size      = *value.read::<f32>(),
            "line_height"    => self.line_height    = *value.read::<LineHeight>(),
            "font_smoothing" => self.font_smoothing = *value.read::<FontSmoothing>(),
            _ => {}
        }
    }

    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer) {
        let font = TextFont {
            font: self.font.handle(server),
            font_size: self.font_size,
            line_height: self.line_height,
            font_smoothing: self.font_smoothing,
        };

        entity.insert(font);
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(TextFontInjector)
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "font"           => self.font           = RawHandle::new(value.to_string()),
            "font_size"      => self.font_size      = f32::from_str(value).unwrap(),
            "line_height"    => self.line_height    = LineHeight::from_str_typed(value).unwrap(),
            "font_smoothing" => self.font_smoothing = FontSmoothing::from_str_typed(value).unwrap(),
            _ => return false,
        }

        true
    }
}