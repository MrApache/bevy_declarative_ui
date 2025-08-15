use std::collections::HashMap;
use bevy::prelude::{Bundle, Component, Entity};
use crate::commands::{UiContext};
use crate::prelude::UiResources;

#[derive(Bundle)]
pub struct UiTemplateBundle {
    pub(crate) context:    UiContext,
    pub(crate) resources:  UiResources,
    pub(crate) tag:        Template,
}

#[derive(Component)]
pub struct Template;

#[derive(Component, Default)]
pub struct Templates {
    ///Entity with (RootId, Resources)
    pub(crate) spawned: HashMap<String, Entity>,
    pub(crate) queue: Vec<TemplateRequest>
}

impl Templates {
    pub fn spawned(&self) -> &HashMap<String, Entity> {
        &self.spawned
    }

    pub fn spawn(&mut self, name: &str, template: impl IntoTemplate) {
        if self.spawned.contains_key(name) {
            panic!("Template \"{}\" already exists", name.to_string());
        }
        let mut template: TemplateRequest = template.into_template();
        template.instance_name = name.to_string();
        self.queue.push(template);
    }

    ///Container property will be ignored
    pub fn spawn_or_insert(&mut self, name: &str, template: impl IntoTemplate) {
        let mut template: TemplateRequest = template.into_template();
        template.instance_name = name.to_string();
        if self.spawned.contains_key(name) {
            template.action = TemplateAction::SpawnOrInsert;
        }
        self.queue.push(template);
    }

    ///Container property will be ignored
    pub fn insert(&mut self, name: &str, template: impl IntoTemplate) {
        if !self.spawned.contains_key(name) {
            panic!("Template \"{}\" does not exists", name.to_string());
        }
        let mut template: TemplateRequest = template.into_template();
        template.instance_name = name.to_string();
        template.action = TemplateAction::Insert;
        self.queue.push(template);
    }
}
pub(crate) enum TemplateAction {
    Spawn,
    SpawnOrInsert,
    ///Replace all resources
    Insert,
}

pub struct TemplateRequest {
    pub(crate) name: String,
    pub(crate) container: String,
    pub(crate) resources: UiResources,

    pub(crate) action: TemplateAction,
    pub(crate) instance_name: String,
}

impl TemplateRequest {
    pub fn new(name: impl Into<String>, container: impl Into<String>, resources: UiResources) -> Self {
        Self {
            name: name.into(),
            container: container.into(),
            resources,

            action: TemplateAction::Spawn,
            instance_name: String::new(),
        }
    }
}


pub trait IntoTemplate {
    fn into_template(self) -> TemplateRequest;
}