use std::str::FromStr;
use bevy::asset::AssetServer;
use bevy::prelude::{Color, EntityCommands, Image, ImageNode, NodeImageMode, Rect};
use crate::injector::Injector;
use crate::prelude::{Extractor, FromStrTyped, ValueStorage};
use crate::xml_component::XmlComponent;
use crate::raw_handle::RawHandle;

pub struct ImageNodeInjector;
impl Injector for ImageNodeInjector {
    fn inject_value(&self,name: &str, value: &ValueStorage, extractor: &mut Extractor, server: &AssetServer) {
        extractor.extract::<ImageNode, _>(|c| {
            match name {
                "flip_x" => c.flip_x = *value.read::<bool>(),
                "flip_y" => c.flip_y = *value.read::<bool>(),
                "color"  => c.color  = *value.read::<Color>(),
                "image"  => c.image  = server.load(value.read::<String>()),
                _ => {},
            }
        });
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImageNodeParser {
    color:  Color,
    rect:   Option<Rect>,
    mode:   NodeImageMode,
    flip_x: bool,
    flip_y: bool,
    image:  RawHandle<Image>,
}

impl XmlComponent for ImageNodeParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        match name {
            "flip_x" => self.flip_x = *value.read::<bool>(),
            "flip_y" => self.flip_y = *value.read::<bool>(),
            "color"  => self.color  = *value.read::<Color>(),
            "image"  => self.image  = RawHandle::new(value.read::<String>().clone()),
            _ => {},
        }
    }

    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer) {
        entity.insert(ImageNode {
            color: self.color,
            image: self.image.handle(server),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            rect: self.rect,
            image_mode: self.mode.clone(),
            texture_atlas: None,
        });
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(ImageNodeInjector)
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "flip_x" => self.flip_x = bool::from_str(value).unwrap(),
            "flip_y" => self.flip_y = bool::from_str(value).unwrap(),
            "image"  => self.image  = RawHandle::new(value.to_string()),
            "color"  => self.color  = Color::from_str_typed(value).unwrap(),
            _ => return false,
        }

        true
    }
}