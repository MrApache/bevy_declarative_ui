use std::collections::HashMap;
use bevy::ecs::system::{SystemId, SystemParam};
use bevy::prelude::*;
use crate::loader::{ParsedTree, XmlAsset};
use crate::parser::{Layouts, Resources};
use crate::{UiDocumentTemplate, XmlLibrary};

#[derive(Component, Clone)]
pub struct UiDocument {
    handle: Handle<XmlAsset>,
}

impl UiDocument {
    pub const fn new(handle: Handle<XmlAsset>) -> Self {
        Self { handle }
    }
}

#[derive(Component)]
pub(crate) struct UiDocumentPrepared;

#[derive(Component, Reflect, Debug, Clone, Default)]
pub(crate) struct UiId(pub String);

pub(crate) struct ResourceCollection<'a> {
    global: &'a Resources,
    local: &'a Resources
}

impl<'a> ResourceCollection<'a> {
    fn new(global: &'a Resources, local: &'a Resources) -> Self {
        Self { global, local }
    }

    fn get(&self, key: &str) -> Option<&String> {
        if let Some(local_value) = self.local.get(key) {
            Some(local_value)
        }
        else if let Some(global_value) = self.global.get(key) {
            Some(global_value)
        }
        else {
            None
        }
    }
}

pub(crate) fn hot_reload(
    mut commands: Commands,
    mut events:   EventReader<AssetEvent<XmlAsset>>,
    documents:    Query<(Entity, &UiDocument)>,
    layouts:      Res<Layouts>,
    library:      Res<XmlLibrary>,
    children:     Query<&Children>,
    server:       Res<AssetServer>,
) {
    events.read().for_each(|ev| {
        let id = match ev {
            AssetEvent::Modified { id } => id,
            _ => {
                return;
            }
        };

        for (e, aa) in documents {
            if !id.eq(&aa.handle.id()) {
                continue;
            }

            if let Ok(children) = children.get(e) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }

            commands.entity(e).clear();
            commands.entity(e).insert(aa.clone());
            commands.entity(e).insert(UiDocumentPrepared);

            if layouts.contains_key(&aa.handle.id()) {
                let mut entity: EntityCommands = commands.entity(e);
                let layout = layouts.get(&aa.handle.id()).unwrap();
                let local = Resources::default();
                let resources = ResourceCollection::new(&layout.resources, &local);
                spawn_layout(
                    &mut entity,
                    &server,
                    &library,
                    &layout.root,
                    &resources,
                );
            }
            else {
                panic!("??");
            }
        }
    });
}

pub(crate) fn spawn_command(
    mut commands: Commands,
    documents:    Query<(Entity, &UiDocument), Without<UiDocumentPrepared>>,
    layouts:      Res<Layouts>,
    server:       Res<AssetServer>,
    library:      Res<XmlLibrary>,
) {
    for (e, document) in documents.iter() {
        if layouts.contains_key(&document.handle.id()) {
            let layout = layouts.get(&document.handle.id()).unwrap();
            let mut entity = commands.entity(e);
            let local = Resources::default();
            let resources = ResourceCollection::new(&layout.resources, &local);
            spawn_layout(
                &mut entity,
                &server,
                &library,
                &layout.root,
                &resources,
            );
            commands.entity(e).insert(UiDocumentPrepared);
        }
    }
}

pub(crate) fn spawn_layout(
    entity:    &mut EntityCommands,
    server:    &AssetServer,
    library:   &XmlLibrary,
    tree:      &ParsedTree,
    resources: &ResourceCollection,
) {
    tree.components.iter().for_each(|c| {
        c.insert_to(entity, server);
    });

    tree.container_properties.iter().for_each(|(name, property)| {
        if name == "id" {
            entity.insert(UiId(resources.get(property).unwrap().clone()));
        }
        else if let Some(function) = library.functions.get(name.as_str()) {
            function.insert_function_tag(resources.get(property).unwrap(), entity);
        }
        else {
            error!("[Container] Unknown attribute: {}", name);
        }
    });

    if let Some(id) = &tree.id {
        entity.insert(UiId(id.clone()));
    }

    let functions = &tree.functions;
    for (name, value) in functions {
        let factory = library.functions.get(name.as_str()).unwrap();
        factory.insert_function_tag(value, entity);
    }

    if tree.containers.is_empty() {
        return;
    }
    let parent = entity.id();
    let mut commands = entity.commands();
    for container in &tree.containers {
        let mut children = commands.spawn_empty();
        spawn_layout(
            &mut children,
            &server,
            &library,
            &container,
            &resources,
        );
        children.insert(ChildOf(parent));
    }
}

#[derive(Component)]
pub(crate) struct UiTemplatePrepared;

pub(crate) fn spawn_template(
    mut commands: Commands,
    mut templates: Query<(Entity, &UiDocumentTemplate), Without<UiTemplatePrepared>>,
    mut query:     Query<(Entity, &UiId)>,
    layouts:       Res<Layouts>,
    library:       Res<XmlLibrary>,
    server:        Res<AssetServer>,
) {
    templates.iter_mut().for_each(|(tmp, template)| {
        if layouts.contains_key(&template.target_layout.id()) {
            for (c, ui_id) in query.iter_mut() {
                if ui_id.0 != template.target_container {
                    continue;
                }
                let layout = layouts.get(&template.target_layout.id()).unwrap();
                let target_template = layout.templates.get(&template.name).unwrap();

                let mut tree = target_template.root.clone();

                let local = &template.resources;
                let resources = ResourceCollection::new(&layout.resources, &local);

                set_component_properties(&mut tree, &resources);

                let mut c_commands = commands.entity(c);
                spawn_layout(
                    &mut c_commands,
                    &server,
                    &library,
                    &tree,
                    &resources
                );

                let mut tmp_commands = commands.entity(tmp);
                tmp_commands.despawn();
                break;
            }
        }
    });
}

fn set_component_properties(tree: &mut ParsedTree, properties: &ResourceCollection) {
    for ap in &tree.properties {
        let value = properties.get(&ap.property).expect(&format!("[Ui Layout Template] Property not found: {}", ap.property));
        for component in &mut tree.components {
            component.parse_attribute(&ap.attribute, value);
        }
    }

    for container in &mut tree.containers {
        set_component_properties(container, properties);
    }
}

#[derive(SystemParam)]
pub struct UiFunctionRegistry<'w, 's> {
    functions: ResMut<'w, UiFunctions>,
    cmd: Commands<'w, 's>,
}

impl<'w, 's> UiFunctionRegistry<'w, 's> {
    pub fn register<S, M>(&mut self, name: impl Into<String>, func: S)
    where
        S: IntoSystem<In<Entity>, (), M> + 'static,
    {
        let id = self.cmd.register_system(func);
        self.functions.register(name, id);
    }
}

#[derive(Resource, Default)]
pub struct UiFunctions {
    map: HashMap<String, SystemId<In<Entity>>>,
}

impl UiFunctions {
    pub fn register(&mut self, key: impl Into<String>, system_id: SystemId<In<Entity>>) {
        let key: String = key.into();
        self.map.insert(key, system_id);
    }

    pub fn maybe_run(&self, key: &String, entity: Entity, cmd: &mut Commands) {
        self.map.get(key)
            .map(|id| {
                cmd.run_system_with(*id, entity);
            })
            .unwrap_or_else(|| error!("[Ui Functions] Function `{key}` is not bound"));
    }
}