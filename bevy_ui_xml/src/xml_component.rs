use std::fmt::Debug;
use bevy::ecs::component::Mutable;
use bevy::prelude::{AssetServer, Component, Entity, EntityCommands, World};
use dyn_clone::DynClone;

pub struct Extractor<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> Extractor<'a> {
    pub(crate) fn new(world: &'a mut World, entity: Entity) -> Self {
        Self { entity, world }
    }

    pub fn extract<T, F>(&mut self, callback: F)
    where
        T: Component<Mutability = Mutable>,
        F: FnOnce(&mut T),
    {
        let mut entity = self.world.entity_mut(self.entity);
        let mut component = entity.get_mut::<T>().unwrap();
        callback(&mut component);
    }
}

pub type XmlComponentFactory = fn() -> Box<dyn XmlComponent>;

pub trait XmlComponent: Send + Sync + 'static + Debug + DynClone {
    fn inject_value(&self, name: &str, value: &str, extractor: &mut Extractor, server: &AssetServer);
    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer);
    fn clear(&mut self) {}
    fn parse_attribute(&mut self, _name: &str, _value: &str) -> bool {false}
    fn parse_nested_element(&mut self, _node: roxmltree::Node) {}
    fn is_nested_element(&self, _node: roxmltree::Node) -> bool { false }
}

pub trait XmlTypeParser: Default {
    fn xml_parse_from_string(value: &str) -> Result<Self, ()> where Self: Sized;
}