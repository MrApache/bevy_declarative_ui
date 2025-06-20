use std::any::type_name;
use std::fmt::Debug;
use bevy::ecs::component::Mutable;
use bevy::prelude::{error, Asset, AssetServer, Component, Entity, EntityCommands, Handle, World};
use dyn_clone::DynClone;
use crate::injector::Injector;
use crate::prelude::{TypedStorage, UntypedStorage};

pub struct ValueStorage<'a> {
    value: &'a Box<dyn UntypedStorage>,
}

impl<'a> ValueStorage<'a> {
    pub fn new(value: &'a Box<dyn UntypedStorage>) -> Self {
        Self { value }
    }

    pub fn read<Type: 'static>(&self) -> &Type {
        self.value.as_any().downcast_ref::<TypedStorage<Type>>().unwrap().get()
    }

    pub fn load<A: Asset>(&self, server: &AssetServer) -> Handle<A> {
        let path: &String = self.read::<String>();
        server.load::<A>(path)
    }
}

pub struct Extractor<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> Extractor<'a> {
    pub(crate) fn new(
        world:  &'a mut World,
        entity: Entity
    ) -> Self {
        Self {entity, world }
    }

    pub fn extract<T, F>(&mut self, callback: F)
    where
        T: Component<Mutability = Mutable>,
        F: FnOnce(&mut T),
    {
        let entity = self.entity;
        let mut entity = self.world.entity_mut(entity);
        let mut component = entity.get_mut::<T>().unwrap();
        callback(&mut component);
    }
}

pub type XmlComponentFactory = fn() -> Box<dyn XmlComponent>;

pub trait XmlComponent: Send + Sync + Debug + DynClone + 'static {
    fn write_value(&mut self, name: &str, value: &ValueStorage);
    fn insert_to(&self, entity: &mut EntityCommands, server: &AssetServer);
    fn as_injector(&self) -> Box<dyn Injector>;
    fn parse_attribute(&mut self, _name: &str, _value: &str) -> bool {false}
}

pub struct MutValueStorage<'a> {
    value: &'a mut Box<dyn UntypedStorage>,
}

impl<'a> MutValueStorage<'a> {
    pub fn new(value: &'a mut Box<dyn UntypedStorage>) -> Self {
        Self { value }
    }

    pub fn write<PType: FromStrTyped>(&mut self, value: &str) {
        let result: Result<PType, String> = PType::from_str_typed(value);
        let value: PType = if result.is_err() {
            error!("{}", result.err().unwrap());
            PType::default()
        } else {
            result.unwrap()
        };
        if let Some(storage) = self.value.as_any_mut().downcast_mut::<TypedStorage<PType>>() {
            storage.set(value)
        }
        else {
            panic!("Type error todo: {}", type_name::<PType>());
        }
    }
}

pub trait FromStrTyped: Default + IsTyped {
    fn from_str_typed(s: &str) -> Result<Self, String>
    where
        Self: Sized;
}

pub trait IsTyped: Send + Sync + 'static {
    fn write_to_storage(&self, value: &str, storage: &mut MutValueStorage);
}