use bevy::prelude::{EntityCommands, AssetServer};
use crate::base::{NodeParser, TextColorParser, TextFontParser, TextParser};
use crate::prelude::Extractor;
use crate::xml_component::XmlComponent;

#[derive(Default, Debug, Clone)]
pub struct TextBundleParser {
    node_parser: NodeParser,
    text_parser: TextParser,
    text_font_parser: TextFontParser,
    text_color_parser: TextColorParser,
}

impl XmlComponent for TextBundleParser {
    fn inject_value(&self, name: &str, value: &str, extractor: &mut Extractor, server: &AssetServer) {
        self.node_parser.inject_value(name, value, extractor, server);
        self.text_parser.inject_value(name, value, extractor, server);
        self.text_color_parser.inject_value(name, value, extractor, server);
        self.text_font_parser.inject_value(name, value, extractor, server);
    }

    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer) {
        self.node_parser.insert_to(entity, server);
        self.text_parser.insert_to(entity, server);
        self.text_color_parser.insert_to(entity, server);
        self.text_font_parser.insert_to(entity, server);
    }

    fn clear(&mut self) {
        self.node_parser.clear();
        self.text_font_parser.clear();
        self.text_color_parser.clear();
        self.text_parser.clear();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        if self.text_parser.parse_attribute(name, value) {
            true
        }
        else if self.text_font_parser.parse_attribute(name, value) {
            true
        }
        else if self.text_color_parser.parse_attribute(name, value) {
            true
        }
        else if self.node_parser.parse_attribute(name, value) {
            true
        }
        else {
            false
        }
    }
}