use std::fmt::Debug;
use bevy::prelude::{AssetServer, EntityCommands};
use dyn_clone::DynClone;

pub type XmlComponentFactory = fn() -> Box<dyn XmlComponent>;

pub trait XmlComponent: Send + Sync + 'static + Debug + DynClone {
    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer);
    fn clear(&mut self);
    fn parse_attribute(&mut self, name: &str, value: &str) -> bool;
    fn parse_nested_element(&mut self, _node: roxmltree::Node) {}
    fn is_nested_element(&self, _node: roxmltree::Node) -> bool { false }
}