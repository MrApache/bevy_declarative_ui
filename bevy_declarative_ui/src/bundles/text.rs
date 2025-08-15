use bevy::prelude::{EntityCommands, AssetServer};
use crate::prelude::{
    XmlComponent,
    Extractor,
    ValueStorage,
    Injector,
    NodeInjector,
    TextColorInjector,
    TextFontInjector,
    TextInjector,
    NodeParser,
    TextColorParser,
    TextFontParser,
    TextParser,
};

pub struct TextBundleInjector;
impl Injector for TextBundleInjector {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, server: &AssetServer) {
        NodeInjector.inject_value(name, value, extractor, server);
        TextInjector.inject_value(name, value, extractor, server);
        TextColorInjector.inject_value(name, value, extractor, server);
        TextFontInjector.inject_value(name, value, extractor, server);
    }
}

#[derive(Default, Debug, Clone)]
pub struct TextBundleParser {
    node_parser: NodeParser,
    text_parser: TextParser,
    text_font_parser: TextFontParser,
    text_color_parser: TextColorParser,
}

impl XmlComponent for TextBundleParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        self.node_parser.write_value(name, value);
        self.text_parser.write_value(name, value);
        self.text_color_parser.write_value(name, value);
        self.text_font_parser.write_value(name, value);
    }

    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer) {
        self.node_parser.insert_to(entity, server);
        self.text_parser.insert_to(entity, server);
        self.text_color_parser.insert_to(entity, server);
        self.text_font_parser.insert_to(entity, server);
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(TextBundleInjector)
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