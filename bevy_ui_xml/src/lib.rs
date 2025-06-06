use std::collections::HashMap;
use bevy::prelude::*;

mod raw_handle;
mod commands;
mod xml_component;
mod loader;
mod types;
mod bundles;
mod base;

use crate::loader::{ParsedTree, UiTemplate};
use crate::commands::{
    hot_reload,
    //observe_interaction,
    observe_on_spawn,
    spawn_command,
    spawn_template,
    UiFunctions,
    UiId
};

pub mod prelude {
    pub use crate::loader::UiLayoutLoader;
    pub use crate::xml_component::*;

    pub use crate::base::*;
    pub use crate::bundles::*;

    pub use crate::raw_handle::RawHandle;
    pub use crate::UiLayout;
    pub use crate::UiDocumentTemplate;
    pub use crate::commands::UiDocument;
    pub use crate::commands::UiFunctionRegistry;
}

#[derive(Asset, TypePath, Debug)]
pub struct UiLayout {
    pub(crate) root: ParsedTree,
    pub(crate) templates: HashMap<String, UiTemplate>,
}

#[derive(Component)]
pub struct UiDocumentTemplate {
    pub name: String,
    pub target_layout: Handle<UiLayout>,
    pub target_container: String,
    pub properties: HashMap<String, String>,
}

pub struct UiXmlPlugin;

impl Plugin for UiXmlPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<UiLayout>();
        app.init_resource::<UiFunctions>();
        app.register_type::<UiId>();

        app.add_systems(PreUpdate, (spawn_command, spawn_template, observe_on_spawn));//, observe_interaction));
        app.add_systems(Update, hot_reload);
    }
}