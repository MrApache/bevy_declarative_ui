use bevy_declarative_ui_parser::values::bindings::BindingKind;
use bevy_declarative_ui_parser::values::bindings::filter::Filters;
use std::collections::HashMap;

#[derive(Default)]
pub struct Required {
    pub ids: Vec<String>,

    ///Resource name, argument name
    pub resources: HashMap<String, String>,

    ///Component name, argument name, filters
    pub components: HashMap<String, (String, Filters)>,

    ///Id, bindings
    pub bindings: HashMap<String, Vec<RequiredBinding>>,

    ///Indicates that function requires asset server
    pub asset_server: bool,
}

pub struct RequiredBinding {
    pub inner: BindingKind,
    pub component: String,
    pub field_name: String,
}
