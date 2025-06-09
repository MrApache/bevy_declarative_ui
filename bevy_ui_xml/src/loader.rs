use bevy::prelude::*;

pub trait AttributeFunction: Sync + Send + 'static {
    fn insert_function_tag(&self, value: &str, entity: &mut EntityCommands);
}

#[derive(Debug, Clone)]
pub(crate) struct AttributeProperty {
    pub(crate) attribute: String,
    pub(crate) property: String,
}