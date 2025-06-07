use std::collections::HashMap;
use bevy::prelude::*;
use crate::base::add_base;
use crate::bundles::add_bundles;

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
    hot_reload,
    spawn_command,
    spawn_template,
    UiFunctions,
    UiId
};
use crate::parser::{parse_xml, Layouts, Resources};
use crate::prelude::{XmlComponent, XmlComponentFactory};

pub mod prelude {
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
        UiDocument,
        UiFunctionRegistry,
        UiFunctions,
    };
}

#[derive(Debug)]
pub struct UiLayout {
    pub(crate) root: ParsedTree,
    pub(crate) templates: HashMap<String, UiTemplate>,
    pub(crate) resources: Resources,
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

pub struct UiXmlPlugin;

impl Plugin for UiXmlPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<XmlAsset>();
        app.init_resource::<UiFunctions>();
        app.init_resource::<Layouts>();
        app.init_asset_loader::<UiXmlLoader>();
        app.register_type::<UiId>();

        app.add_systems(Update, (
            parse_xml,
            hot_reload,
            spawn_command,
            spawn_template,
        ));
    }
}