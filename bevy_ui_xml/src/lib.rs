use std::collections::HashMap;
use bevy::prelude::*;
use crate::base::add_base;
use crate::bundles::add_bundles;

/*
TODO:
  Resources hierarchy:
    in document: Local > Global -> Error
    in template: Spawn > Local > Global -> Error

TODO:
    Refactor tag parsing:
        Custom XML Parser??

TODO:
    Template with document Id

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
mod loader;
mod types;
mod bundles;
mod base;
mod parser;
mod xml_parser;
mod functions;

use crate::loader::{
    AttributeFunction,
};

use crate::commands::{
    asset_event_reader,
    spawn_command,
    spawn_template,
    sync_local_resources,
    GlobalResources,
    UiContainerId
};
use crate::functions::UiFunctions;
use crate::parser::CompiledLayout;
use crate::prelude::{UiDocument, UiDocumentId, XmlComponent, XmlComponentFactory};
use crate::xml_parser::{Resources, XmlLayout, XmlLoader};

pub mod prelude {
    pub use crate::SyncSet;
    pub use crate::loader::{
        AttributeFunction,
    };
    pub use crate::xml_component::*;
    pub use crate::xml_parser::{
        XmlLayout,
        Resources
    };

    pub use crate::functions::*;

    pub use crate::base::*;
    pub use crate::bundles::*;

    pub use crate::raw_handle::RawHandle;
    pub use crate::UiDocumentTemplate;
    pub use crate::commands::{
        UiDocumentId,
        UiDocument,
    };
}

#[derive(Resource, Deref, DerefMut, Default)]
pub(crate) struct Layouts(HashMap<AssetId<XmlLayout>, CompiledLayout>);

#[derive(Component)]
pub struct UiDocumentTemplate {
    pub name: String,
    pub target_layout: Handle<XmlLayout>,
    pub target_container: String,
    pub resources: Resources,
}

#[derive(Resource)]
pub struct XmlLibrary {
    factories: HashMap<&'static str, XmlComponentFactory>,
    functions: HashMap<&'static str, Box<dyn AttributeFunction>>,
}

impl Default for XmlLibrary {
    fn default() -> Self {
        let mut loader = XmlLibrary {
            factories: HashMap::new(),
            functions: HashMap::new()
        };

        add_base(&mut loader);
        add_bundles(&mut loader);

        loader
    }
}

impl XmlLibrary {
    pub fn add_component(&mut self, name: &'static str, factory: XmlComponentFactory) {
        self.factories.insert(name, factory);
    }

    pub fn add_function<T: AttributeFunction>(&mut self, name: &'static str, factory: T) {
        self.functions.insert(name, Box::new(factory));
    }

    pub(crate) fn get_component(&self, tag: &str) -> Box<dyn XmlComponent> {
        if !self.factories.contains_key(tag) {
            panic!("[Ui layout] Unknown tag: {}", tag)
        }

        self.factories.get(tag).unwrap()()
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyncSet {
    Functions,
    SyncLocalResources,
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
        app.register_type::<UiDocument>();
        app.register_type::<UiDocumentId>();

        app.add_systems(Update, (
            asset_event_reader,
            spawn_command,
            spawn_template,
        ));

        app.add_systems(Last, sync_local_resources.in_set(SyncSet::SyncLocalResources));

        app.configure_sets(PostUpdate, SyncSet::Functions);
        app.configure_sets(Last, SyncSet::SyncLocalResources);
    }
}