use std::collections::HashMap;
use bevy::ecs::system::{SystemId, SystemParam};
use bevy::prelude::*;
use crate::loader::{ParsedTree, XmlAsset};
use crate::parser::Layouts;
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
                spawn_layout(&mut entity, &layouts.get(&aa.handle.id()).unwrap().root, &server, &library);
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
            let mut entity = commands.entity(e);
            spawn_layout(&mut entity, &layouts.get(&document.handle.id()).unwrap().root, &server, &library);
            commands.entity(e).insert(UiDocumentPrepared);
        }
    }
}

pub(crate) fn spawn_layout(
    entity:  &mut EntityCommands,
    tree:    &ParsedTree,
    server:  &AssetServer,
    library: &XmlLibrary,
) {
    tree.components.iter().for_each(|c| {
        c.insert_to(entity, server);
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
        spawn_layout(&mut children, &container, server, library);
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
                let target_template = layouts.get(&template.target_layout.id()).unwrap()
                    .templates.get(&template.name).unwrap();
                let mut tree = target_template.root.clone();

                set_properties(&mut tree, &template.properties);

                let mut c_commands = commands.entity(c);
                spawn_layout(&mut c_commands, &tree, &server, &library);

                let mut tmp_commands = commands.entity(tmp);
                tmp_commands.despawn();
                break;
            }
        }
    });
}

fn set_properties(tree: &mut ParsedTree, properties: &HashMap<String, String>) {
    for ap in &tree.properties {
        let value = properties.get(&ap.property).unwrap();
        for component in &mut tree.components {
            component.parse_attribute(&ap.attribute, value);
        }
    }

    for container in &mut tree.containers {
        set_properties(container, properties);
    }
}

#[derive(SystemParam)]
pub struct UiFunctionRegistry<'w, 's> {
    functions: ResMut<'w, UiFunctions>,
    cmd: Commands<'w, 's>,
}

//pub type SpawnFunction = dyn Fn(EntityCommands) + Send + Sync + 'static;

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
            .unwrap_or_else(|| warn!("function `{key}` is not bound"));
    }
}