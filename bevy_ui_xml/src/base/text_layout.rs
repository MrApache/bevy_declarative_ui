use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::prelude::Extractor;
use crate::xml_component::XmlComponent;

#[derive(Default, Clone, Debug)]
pub struct TextLayoutParser {
    text_layout: TextLayout,
}

impl XmlComponent for TextLayoutParser {
    fn inject_value(&self, name: &str, value: &str, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<TextLayout, _>(|c| {
            match name {
                "justify"   => c.justify   = parse_justify_text(value),
                "linebreak" => c.linebreak = parse_line_break(value),
                _ => {},
            }
        })
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(self.text_layout.clone());
    }

    fn clear(&mut self) {
        self.text_layout = TextLayout::default();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "justify" => self.text_layout.justify = parse_justify_text(value),
            "linebreak" => self.text_layout.linebreak = parse_line_break(value),
            _ => return false,
        }

        true
    }
}

fn parse_justify_text(value: &str) -> JustifyText {
    match value {
        "Left" => JustifyText::Left,
        "Center" => JustifyText::Center,
        "Right" => JustifyText::Right,
        "Justified" => JustifyText::Justified,
        _ => {
            error!("[TextLayout] Unknown justify text value: {}", value);
            JustifyText::default()
        }
    }
}

fn parse_line_break(value: &str) -> LineBreak {
    match value {
        "WordBoundary" => LineBreak::WordBoundary,
        "AnyCharacter" => LineBreak::AnyCharacter,
        "WordOrCharacter" => LineBreak::WordOrCharacter,
        "NoWrap" => LineBreak::NoWrap,
        _ => {
            error!("[TextLayout] Unknown line break value: {}", value);
            LineBreak::default()
        }
    }
}
