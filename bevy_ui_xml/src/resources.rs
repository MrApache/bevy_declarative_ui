use std::default::Default;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use bevy::asset::AssetId;
use bevy::prelude::{Component, Resource};
use crate::prelude::XmlLayout;

pub trait UntypedStorage: Send + Sync + 'static   {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn changed(&self) -> bool;
    fn checked(&mut self);
    fn set_changed(&mut self);
}

#[derive(Default, Clone)]
pub struct TypedStorage<T: 'static> {
    value: T,
    changed: bool,
}

impl<T: 'static> TypedStorage<T> {
    pub(crate) fn set(&mut self, value: T) {
        self.value = value;
        self.changed = true;
    }

    pub(crate) fn get(&self) -> &T{
        &self.value
    }
}

impl<T: 'static + Send + Sync> UntypedStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn changed(&self) -> bool {
        self.changed
    }

    fn checked(&mut self) {
        self.changed = false;
    }

    fn set_changed(&mut self) {
        self.changed = true;
    }
}

pub(crate) struct Storage {
    pub storage: Box<dyn UntypedStorage>,
    pub is_inherit: bool,
    pub overridden: bool,
}

impl Debug for Storage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("is_inherit", &self.is_inherit)
            .field("overridden", &self.overridden)
            .finish()
    }
}

impl Storage {
    pub fn new(storage: Box<dyn UntypedStorage>, is_inherit: bool) -> Self {
        Self {
            storage,
            is_inherit,
            overridden: false,
        }
    }
}

fn downcast_mut<T: 'static>(b: &mut Box<dyn UntypedStorage>) -> &mut TypedStorage<T> {
    b.as_any_mut().downcast_mut::<TypedStorage<T>>().unwrap()
}

fn downcast<T: 'static>(b: &Box<dyn UntypedStorage>) -> &TypedStorage<T> {
    b.as_any().downcast_ref::<TypedStorage<T>>().unwrap()
}

pub trait PropertyType: Default {
    type Type: Default + Send + Sync + 'static;
}

pub type StorageFactory = fn() -> Box<dyn UntypedStorage>;


#[derive(Resource, Default)]
pub struct GlobalResources {
    pub(crate) storage: HashMap<AssetId<XmlLayout>, UiResources>,
    pub(crate) changed: bool,
}

impl GlobalResources {
    pub fn set_property<Key: 'static + PropertyType>(&mut self, id: &AssetId<XmlLayout>, value: Key::Type) {
        self.storage.get_mut(id).unwrap().set_property::<Key>(value);
    }
}

#[derive(Component, Default)]
pub struct UiResources {
    pub(crate) properties: HashMap<TypeId, Storage>,
}

impl UiResources {
    pub(crate) fn get_property(&self, type_id: TypeId) -> Option<&Storage> {
        self.properties.get(&type_id)
    }

    pub(crate) fn add_property_internal(&mut self, type_id: TypeId, storage: Storage) {
        self.properties.insert(type_id, storage);
    }

    pub fn add_property<Key: 'static + PropertyType>(&mut self, value: Key::Type) {
        let mut storage: TypedStorage<Key::Type> = TypedStorage::default();
        storage.set(value);
        let storage: Storage = Storage::new(Box::new(storage), false);
        self.properties.insert(TypeId::of::<Key>(), storage);
    }

    pub(crate) fn take_changed(&mut self) -> Vec<(TypeId, Storage)>{
        let changed_keys: Vec<TypeId> = self
            .properties
            .iter()
            .filter_map(|(key, storage)| {
                if storage.storage.changed() {
                    if storage.is_inherit && !storage.overridden {
                        return None;
                    } 
                    Some(*key)
                } else {
                    None
                }
            })
            .collect();

        let mut buffer = vec![];
        
        for key in changed_keys {
            if let Some(mut storage) = self.properties.remove(&key) {
                storage.storage.checked();
                buffer.push((key, storage));
            }
        }
        
        buffer
    }

    pub fn set_property<Key: 'static + PropertyType>(&mut self, value: Key::Type) {
        let untyped_storage: &mut Storage = self.properties.get_mut(&TypeId::of::<Key>()).unwrap();
        debug_assert!(!untyped_storage.is_inherit, "For overriding '{}' use 'override_property()' instead", type_name::<Key>());
        let storage: &mut TypedStorage<Key::Type> = downcast_mut::<Key::Type>(&mut untyped_storage.storage);
        storage.set(value);
    }

    pub fn override_property<Key: 'static + PropertyType>(&mut self, value: Option<Key::Type>) {
        let untyped_storage: &mut Storage = self.properties.get_mut(&TypeId::of::<Key>()).unwrap();
        debug_assert!(untyped_storage.is_inherit, "For non inherit properties use 'set_property()' instead'");
        let storage: &mut TypedStorage<Key::Type> = downcast_mut::<Key::Type>(&mut untyped_storage.storage);
        if value.is_some() {
            untyped_storage.overridden = true;
            storage.set(value.unwrap());
        }
        else {
            untyped_storage.overridden = false;
            storage.set(Key::Type::default());
        }
    }

    pub fn read_property<Key: 'static + PropertyType>(&self) -> &Key::Type {
        let untyped_storage: &Storage = self.properties.get(&TypeId::of::<Key>()).unwrap();
        &downcast::<Key::Type>(&untyped_storage.storage).value
    }
}


#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use bevy::prelude::{Display, Val};
    use crate::prelude::{PropertyType, UiResources, TypedStorage};
    use crate::resources::{downcast, Storage};

    //struct Node.left
    #[derive(Default)]
    struct Left;
    impl PropertyType for Left {
        type Type = Val;
    }

    //struct Node.display
    #[derive(Default)]
    struct Visible;
    impl PropertyType for Visible {
        type Type = Display;
    }

    #[test]
    fn test() {
        let mut resources: UiResources = UiResources::default();
        assert_eq!(resources.properties.len(), 0);

        resources.add_property_internal(TypeId::of::<Left>(), Storage::new(Box::<TypedStorage<Val>>::new(TypedStorage::default()), false));
        resources.add_property_internal(TypeId::of::<Visible>(), Storage::new(Box::<TypedStorage<Display>>::new(TypedStorage::default()), false));

        assert_eq!(resources.properties.len(), 2);

        resources.set_property::<Left>(Val::Px(42.0));
        resources.set_property::<Visible>(Display::Flex);

        assert_eq!(resources.read_property::<Left>(), &Val::Px(42.0));
        assert_eq!(resources.read_property::<Visible>(), &Display::Flex);

        assert_eq!(downcast::<Display>(&resources.get_property(TypeId::of::<Visible>()).unwrap().storage).value, Display::Flex);
        assert_eq!(downcast::<Val>(&resources.get_property(TypeId::of::<Left>()).unwrap().storage).value, Val::Px(42.0));
    }

    #[test]
    fn iter_changed() {
        let mut resources: UiResources = UiResources::default();
        resources.add_property_internal(TypeId::of::<Left>(), Storage::new(Box::<TypedStorage<Val>>::new(TypedStorage::default()), false));
        resources.add_property_internal(TypeId::of::<Visible>(), Storage::new(Box::<TypedStorage<Display>>::new(TypedStorage::default()), false));
        resources.set_property::<Left>(Val::Px(42.0));


        assert_eq!(resources.properties.len(), 2);
        let buffer = resources.take_changed();
        assert_eq!(resources.properties.len(), 1);

        for (id, storage) in buffer {
            resources.add_property_internal(id, storage);
        }
        assert_eq!(resources.properties.len(), 2);
    }
}