use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8};
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use crate::base::add_base;
use crate::bundles::add_bundles;
use crate::functions::UiFunctions;
use crate::parser::CompiledLayout;
use crate::xml_parser::{XmlLayout, XmlLoader};
use crate::commands::{asset_event_reader, spawn_command, template_actions, sync_resources, UiContainerId, UiContext};
use crate::prelude::{
    add_base_types, AttributeCallback, Callbacks, CallbacksArguments, FromStrTyped, GlobalResources, IsTyped, PropertyType, StorageFactory, XmlComponent, XmlComponentFactory
};

/*
TODO:
    Fast template access
    Template presets + code generation
    Generate template code
    Fix UiDocumentPrepared
    0-1 Container per Template
    Functions in Resources
    Support properties generation in runtime (hot reload only)
    make split property trait into global and local
    Entity validation

TODO:
    Error handling
    Beautiful error print
        Print with
            Layout name
            Tag
            Attribute
*/

mod raw_handle;
mod commands;
mod xml_component;
mod bundles;
mod base;
mod parser;
mod xml_parser;
mod functions;
mod resources;
mod injector;
mod templates;
mod test;

pub mod prelude {
    pub use crate::functions::*;
    pub use crate::base::*;
    pub use crate::bundles::*;
    pub use crate::xml_component::*;
    pub use crate::raw_handle::RawHandle;
    pub use crate::XmlLibrary;
    pub use crate::xml_parser::XmlLayout;
    pub use crate::injector::*;
    pub use crate::templates::*;
    pub use crate::resources::*;
    pub use crate::commands::*;
}

#[derive(Component)]
pub struct Template0 {
    pub index: u8,
}

static TEMPLATE0_INDICES: AtomicU8 = AtomicU8::new(0);

fn xd(
    mut cmp0: Query<(&mut Node, &mut ImageNode), With<RuntimeTemplate0>>,
    mut cmp1: Query<&mut AseSlice, With<RuntimeId5>>,
    mut cmp2: Query<&mut Counter, With<RuntimeId6>>,
    mut cmp3: Query<&mut Node, With<RuntimeId7>>,

) {

    cmp0.iter()
        .zip(cmp1.iter_mut())
        .zip(cmp2.iter_mut())
        .zip(cmp3.iter_mut()).for_each(|(((cmp0, cmp1), cmp2), cmp3)| {

    });
}

#[derive(Component)]
pub struct AseSlice;
#[derive(Component)]
pub struct Counter;
#[derive(Component)]
pub struct RuntimeId5;
#[derive(Component)]
pub struct RuntimeId6;
#[derive(Component)]
pub struct RuntimeId7;
#[derive(Component)]
pub struct RuntimeTemplate0;


fn test(
    query0: Query<&Node>,
    query1: Query<&ImageNode>,
    query2: Query<&Interaction>,
) {
    query0.iter()
        .zip(query1.iter())
        .zip(query2.iter())
        .for_each(|x| {

        })
}

#[derive(Resource, Deref, DerefMut, Default)]
pub(crate) struct Layouts(HashMap<AssetId<XmlLayout>, CompiledLayout>);

#[derive(Resource)]
pub struct XmlLibrary {
    factories: HashMap<&'static str, XmlComponentFactory>,
    functions: HashMap<&'static str, Box<dyn AttributeCallback>>,
    /// Layout path | Type Name -> Type Id
    storages:  HashMap<&'static str, HashMap<String, (TypeId, StorageFactory)>>,
    types:     HashMap<&'static str, Box<dyn IsTyped>>
}

impl Default for XmlLibrary {
    fn default() -> Self {
        let mut loader = XmlLibrary {
            factories: HashMap::new(),
            functions: HashMap::new(),
            storages:  HashMap::new(),
            types:     HashMap::new(),
        };

        add_base(&mut loader);
        add_bundles(&mut loader);

        add_base_types(&mut loader);

        loader
    }
}

impl XmlLibrary {
    pub fn empty() -> Self {
        Self {
            factories: HashMap::new(),
            functions: HashMap::new(),
            storages:  HashMap::new(),
            types:     HashMap::new(),
        }
    }

    pub fn add_component(&mut self, name: &'static str, factory: XmlComponentFactory) {
        self.factories.insert(name, factory);
    }

    pub fn add_function<T: AttributeCallback>(&mut self, name: &'static str, factory: T) {
        self.functions.insert(name, Box::new(factory));
    }

    pub fn add_property<T: PropertyType + 'static>(&mut self, path: &'static str, property_name: impl Into<String>, factory: StorageFactory) {
        if !self.storages.contains_key(path) {
            self.storages.insert(path, HashMap::new());
        }
        self.storages.get_mut(path).unwrap().insert(property_name.into(), (TypeId::of::<T>(), factory));
    }

    pub fn add_type<T: FromStrTyped>(&mut self, name: &'static str) {
        self.types.insert(name, Box::new(T::default()));
    }

    pub(crate) fn get_component(&self, tag: &str) -> Box<dyn XmlComponent> {
        if !self.factories.contains_key(tag) {
            panic!("[Ui layout] Unknown tag: {}", tag)
        }

        self.factories.get(tag).unwrap()()
    }
}

pub struct UiXmlPlugin;

impl Plugin for UiXmlPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<XmlLayout>();
        app.init_resource::<UiFunctions>();
        app.init_resource::<Layouts>();
        app.init_resource::<GlobalResources>();
        app.init_asset_loader::<XmlLoader>();

        app.register_type::<UiContainerId>();
        app.register_type::<UiContext>();
        app.register_type::<Callbacks>();
        app.register_type::<CallbacksArguments>();

        app.add_systems(Update, (
            asset_event_reader,
            spawn_command,
            template_actions,
        ));

        app.add_systems(Last, sync_resources);
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::LoadState;
    use bevy::prelude::*;
    use crate::{UiXmlPlugin, XmlLibrary};
    use crate::prelude::XmlLayout;

    fn setup(library: XmlLibrary) -> App {
        let mut app: App = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(UiXmlPlugin);
        app.insert_resource(library);
        app
    }

    #[test]
    fn load_correct_xml() {
        let mut app = setup(XmlLibrary::default());
        let assets: &AssetServer = app.world().resource::<AssetServer>();
        let handle: Handle<XmlLayout> = assets.load("correct.xml");
        update_for(&mut app, 2.0);
        let state = app.world().resource::<AssetServer>().get_load_state(&handle).unwrap();
        assert!(matches!(state, LoadState::Loaded));
    }

    macro_rules! counter {
        () => {
            use crate::prelude::*;
            static INJECTION_COUNTER: AtomicU32 = AtomicU32::new(0);

            #[derive(Component, Default)]
            struct Counter {
                value: u32
            }

            #[derive(Default, Debug, Clone)]
            struct CounterParser {
                value: u32
            }

            struct CounterInjector;
            impl Injector for CounterInjector {
                fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, _: &AssetServer) {
                    extractor.extract::<Counter, _>(|x| {
                        match name {
                            "value" => x.value = *value.read::<u32>(),
                            _ => {}
                        }
                    });

                    INJECTION_COUNTER.fetch_add(1, Ordering::SeqCst);
                }
            }

            impl XmlComponent for CounterParser {
                fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
                    entity.insert(Counter {
                        value: self.value
                    });
                }

                fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
                    match name {
                        "value" => self.value = value.parse().unwrap(),
                        _ => return false,
                    }
                    true
                }

                fn write_value(&mut self, _: &str, _: &ValueStorage) {}

                fn as_injector(&self) -> Box<dyn Injector> {
                    Box::new(CounterInjector)
                }
            }

            #[derive(Default)]
            pub struct CountType;
            impl PropertyType for CountType {
                type Type = u32;
            }
        };
    }

    mod local_res_injector_count_test_1 {
        use crate::prelude::{PropertyType, TypedStorage, ValueStorage};
        use std::sync::atomic::{AtomicU32, Ordering};
        use bevy::asset::{AssetServer, Handle};
        use bevy::prelude::{Component, EntityCommands};
        use crate::commands::UiDocumentBundle;
        use crate::prelude::{Extractor, XmlComponent, XmlLayout};
        use crate::tests::{setup, update_for};
        use crate::XmlLibrary;

        counter!();

        #[test]
        fn injection_count_1() {
            let mut library: XmlLibrary = XmlLibrary::empty();
            library.add_type::<u32>("u32");
            library.add_property::<CountType>("injection_count_1", "Count", || Box::<TypedStorage<u32>>::new(TypedStorage::default()));
            library.add_component("Counter", || Box::new(CounterParser::default()));
            let mut app = setup(library);

            let assets: &AssetServer = app.world().resource::<AssetServer>();
            let handle: Handle<XmlLayout> = assets.load("injection_count_1.xml");
            let entity = app.world_mut().spawn(UiDocumentBundle::new(handle)).id();
            update_for(&mut app, 1.0);

            assert_eq!(app.world().get_entity(entity).unwrap().get::<Counter>().unwrap().value, 100);

            let value = INJECTION_COUNTER.load(Ordering::SeqCst);
            assert_eq!(value, 1);
        }
    }

    mod local_res_injector_count_test_2 {
        use crate::PropertyType;
        use crate::prelude::{TypedStorage, ValueStorage};
        use std::sync::atomic::{AtomicU32, Ordering};
        use bevy::asset::{AssetServer, Handle};
        use bevy::prelude::{Component, EntityCommands, QueryState};
        use crate::commands::UiDocumentBundle;
        use crate::prelude::{Extractor, XmlComponent, XmlLayout};
        use crate::tests::{setup, update_for};
        use crate::XmlLibrary;

        counter!();

        #[test]
        fn injection_count_10() {
            let mut library: XmlLibrary = XmlLibrary::empty();
            library.add_type::<u32>("u32");
            library.add_property::<CountType>("injection_count_10", "Count", || Box::<TypedStorage<u32>>::new(TypedStorage::default()));
            library.add_component("Counter", || Box::new(CounterParser::default()));
            let mut app = setup(library);

            let assets: &AssetServer = app.world().resource::<AssetServer>();
            let handle: Handle<XmlLayout> = assets.load("injection_count_10.xml");
            app.world_mut().spawn(UiDocumentBundle::new(handle));
            update_for(&mut app, 1.0);

            let mut query: QueryState<&Counter> = app.world_mut().query::<&Counter>();
            query.iter(app.world()).for_each(|counter| {
                assert_eq!(counter.value, 1000);
            });

            let value = INJECTION_COUNTER.load(Ordering::SeqCst);
            assert_eq!(value, 10);
        }
    }

    mod global_res_injector_count_test_1 {
        use crate::PropertyType;
        use crate::prelude::{TypedStorage, ValueStorage};
        use std::sync::atomic::{AtomicU32, Ordering};
        use bevy::asset::{AssetServer, Handle};
        use bevy::prelude::{App, Component, EntityCommands, QueryState};
        use crate::commands::{UiDocumentBundle};
        use crate::prelude::{Extractor, XmlComponent, XmlLayout};
        use crate::tests::{setup, update_for};
        use crate::XmlLibrary;

        counter!();

        #[test]
        fn test() {
            let mut library: XmlLibrary = XmlLibrary::empty();
            library.add_type::<u32>("u32");
            library.add_property::<CountType>("global_res", "Count", || Box::<TypedStorage<u32>>::new(TypedStorage::default()));
            library.add_component("Counter", || Box::new(CounterParser::default()));
            let mut app: App = setup(library);

            let assets: &AssetServer = app.world().resource::<AssetServer>();
            let handle: Handle<XmlLayout> = assets.load("global_res.xml");
            app.world_mut().spawn(UiDocumentBundle::new(handle.clone()));
            update_for(&mut app, 1.0);
            let mut query: QueryState<&Counter> = app.world_mut().query::<&Counter>();
            query.iter(app.world()).for_each(|counter| {
                assert_eq!(counter.value, 1000);
            });

            let mut global_resources = app.world_mut().resource_mut::<GlobalResources>();
            global_resources.set_property::<CountType>(&handle.id(), 999);
            update_for(&mut app, 1.0);
            query.iter(app.world()).for_each(|counter| {
                assert_eq!(counter.value, 999);
            });

            let value: u32 = INJECTION_COUNTER.load(Ordering::SeqCst);
            assert_eq!(value, 10);
        }
    }

    mod global_local_res_injector_count_test_1 {
        use crate::PropertyType;
        use crate::prelude::{TypedStorage, UiDocumentBundle, ValueStorage};
        use std::sync::atomic::{AtomicU32, Ordering};
        use bevy::asset::{AssetServer, Handle};
        use bevy::prelude::{App, Component, EntityCommands, QueryState};
        use crate::prelude::{Extractor, XmlComponent, XmlLayout};
        use crate::tests::{setup, update_for};
        use crate::XmlLibrary;

        counter!();

        #[test]
        fn test() {
            let mut library: XmlLibrary = XmlLibrary::empty();
            library.add_type::<u32>("u32");
            library.add_property::<CountType>("global_local_res", "Count", || Box::<TypedStorage<u32>>::new(TypedStorage::default()));
            library.add_component("Counter", || Box::new(CounterParser::default()));
            let mut app: App = setup(library);

            let assets: &AssetServer = app.world().resource::<AssetServer>();
            let handle: Handle<XmlLayout> = assets.load("global_local_res.xml");
            app.world_mut().spawn(UiDocumentBundle::new(handle));
            update_for(&mut app, 1.0);

            let mut query: QueryState<&Counter> = app.world_mut().query::<&Counter>();
            query.iter(app.world()).for_each(|counter| {
                assert_eq!(counter.value, 100);
            });

            let value: u32 = INJECTION_COUNTER.load(Ordering::SeqCst);
            assert_eq!(value, 10);
        }
    }

    fn update_for(app: &mut App, seconds: f32) {
        let mut elapsed_time = 0.0;
        while elapsed_time < seconds {
            app.update();
            elapsed_time += app.world().resource::<Time>().delta_secs() - f32::EPSILON;
        }
    }
}