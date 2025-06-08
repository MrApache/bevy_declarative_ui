use std::str::FromStr;
use bevy::asset::AssetServer;
use bevy::log::error;
use bevy::prelude::{EntityCommands, Font, TextFont};
use bevy::text::{FontSmoothing, LineHeight};
use crate::prelude::Extractor;
use crate::xml_component::XmlComponent;
use crate::raw_handle::RawHandle;

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
    fn inject_value(&self, name: &str, value: &str, extractor: &mut Extractor, server: &AssetServer) {
        extractor.extract::<TextFont, _>(|c| {
            match name {
                "font"           => c.font           = server.load(value),
                "font_size"      => c.font_size      = f32::from_str(value).unwrap(),
                "line_height"    => c.line_height    = parse_line_height(value),
                "font_smoothing" => c.font_smoothing = parse_font_smoothing(value),
                _ => {}
            }
        });
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

    fn clear(&mut self) {
        let s = Self::default();
        self.font = s.font;
        self.line_height = s.line_height;
        self.font_size = s.font_size;
        self.font_smoothing = s.font_smoothing;
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "font"           => self.font           = RawHandle::new(value.to_string()),
            "font_size"      => self.font_size      = f32::from_str(value).unwrap(),
            "line_height"    => self.line_height    = parse_line_height(value),
            "font_smoothing" => self.font_smoothing = parse_font_smoothing(value),
            _ => return false,
        }

        true
    }
}

fn parse_line_height(value: &str) -> LineHeight {
    let s = value.trim();

    if let Some(num_str) = s.strip_suffix("px") {
        match f32::from_str(num_str.trim()) {
            Ok(value) => LineHeight::Px(value),
            Err(_) => {
                error!("Failed to parse number before `px`: {:?}", num_str);
                LineHeight::default()
            }
        }
    } else if let Some(num_str) = s.strip_suffix("rl") {
        match f32::from_str(num_str.trim()) {
            Ok(value) => LineHeight::RelativeToFont(value),
            Err(_) => {
                error!("Failed to parse number before `rl`: {:?}", num_str);
                LineHeight::default()
            }
        }
    } else {
        error!(
                "String `{}` does not match the pattern `<number>px` or `<number>rl`",
                s
            );
        LineHeight::default()
    }
}

fn parse_font_smoothing(value: &str) -> FontSmoothing {
    match value {
        "None" => FontSmoothing::None,
        "AntiAliased" => FontSmoothing::AntiAliased,
        _ => {
            error!("[Text] Unknown font smoothing value: {}", value);
            FontSmoothing::default()
        }
    }
}