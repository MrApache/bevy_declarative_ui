use std::str::FromStr;
use bevy::asset::AssetServer;
use bevy::prelude::{Color, EntityCommands, Image, ImageNode, NodeImageMode, Rect};
use crate::xml_component::XmlComponent;
use crate::raw_handle::RawHandle;
use crate::types::color_str;

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

    fn clear(&mut self) {
        let s = ImageNode::default();
        self.color = s.color;
        self.rect = s.rect;
        self.mode = s.image_mode;
        self.flip_x = s.flip_x;
        self.flip_y = s.flip_y;
        self.image = RawHandle::default();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "flip_x" => self.flip_x = bool::from_str(value).unwrap(),
            "flip_y" => self.flip_y = bool::from_str(value).unwrap(),
            "image"  => self.image  = RawHandle::new(value.to_string()),
            "color"  => self.color  = color_str(value),
            _ => return false,
        }

        true
    }
}