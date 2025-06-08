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

use crate::loader::{
    AttributeFunction,
    ParsedTree,
    UiTemplate,
    UiXmlLoader,
    XmlAsset
};

use crate::commands::{
    asset_event_reader,
    spawn_command,
    spawn_template,
    sync_local_resources,
    UiFunctions,
    UiId
};
use crate::parser::Resources;
use crate::prelude::{UiDocument, UiDocumentId, XmlComponent, XmlComponentFactory};

pub mod prelude {
    pub use crate::SyncSet;
    pub use crate::parser::Resources;
    pub use crate::loader::{
        UiXmlLoader,
        AttributeFunction,
    };
    pub use crate::xml_component::*;
    pub use crate::loader::XmlAsset;

    pub use crate::base::*;
    pub use crate::bundles::*;

    pub use crate::raw_handle::RawHandle;
    pub use crate::UiLayout;
    pub use crate::UiDocumentTemplate;
    pub use crate::commands::{
        UiDocumentId,
        UiDocument,
        UiFunctionRegistry,
        UiFunctions,
    };
}

#[derive(Resource, Deref, DerefMut, Default)]
pub(crate) struct Layouts(HashMap<AssetId<XmlAsset>, UiLayout>);

#[derive(Debug)]
pub struct UiLayout {
    pub(crate) root: ParsedTree,
    pub(crate) templates: HashMap<String, UiTemplate>,
    pub(crate) global: Resources,
    pub(crate) local: Resources,
}

#[derive(Component)]
pub struct UiDocumentTemplate {
    pub name: String,
    pub target_layout: Handle<XmlAsset>,
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
        app.init_asset::<XmlAsset>();
        app.init_resource::<UiFunctions>();
        app.init_resource::<Layouts>();
        app.init_asset_loader::<UiXmlLoader>();

        app.register_type::<UiId>();
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