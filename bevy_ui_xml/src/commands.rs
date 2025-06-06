use std::collections::HashMap;
use bevy::ecs::component::Mutable;
use bevy::ecs::system::{SystemId, SystemParam};
use bevy::prelude::*;
use crate::loader::ParsedTree;
use crate::{UiDocumentTemplate, UiLayout};

#[derive(Component, Clone)]
pub struct UiDocument {
    handle: Handle<UiLayout>,
}

impl UiDocument {
    pub const fn new(handle: Handle<UiLayout>) -> Self {
        Self { handle }
    }
}

#[derive(Component)]
pub(crate) struct UiDocumentPrepared;

#[derive(Component, Reflect, Debug, Clone, Default)]
pub(crate) struct UiId(pub String);

pub(crate) fn hot_reload(
    mut commands: Commands,
    mut events:   EventReader<AssetEvent<UiLayout>>,
    layouts:      Query<(Entity, &UiDocument)>,
    children:     Query<&Children>,
    assets:       Res<Assets<UiLayout>>,
    server:       Res<AssetServer>,
) {
    events.read().for_each(|ev| {
        let id = match ev {
            AssetEvent::Modified { id } => id,
            _ => {
                return;
            }
        };

        for (e, aa) in layouts {
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

            let layout = assets.get(&aa.handle);
            if let Some(layout) = layout {
                let mut entity = commands.entity(e);
                spawn_layout(&mut entity, &layout.root, &server);
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
    assets:       Res<Assets<UiLayout>>,
    server:       Res<AssetServer>,
) {
    for (e, document) in documents.iter() {
        let layout = assets.get(&document.handle);
        if let Some(layout) = layout {
            let mut entity = commands.entity(e);
            spawn_layout(&mut entity, &layout.root, &server);
            commands.entity(e).insert(UiDocumentPrepared);
        }
        else {
            return;
        }
    }
}

pub(crate) fn spawn_layout(
    entity: &mut EntityCommands,
    tree:   &ParsedTree,
    server: &AssetServer,
) {
    tree.components.iter().for_each(|c| {
        c.insert_to(entity, server);
    });

    if let Some(id) = &tree.id {
        entity.insert(UiId(id.clone()));
    }

    let functions = &tree.functions;

    if let Some(on_spawn) = &functions.on_spawn_fn {
        entity.insert(UiSpawnFnReq {
            fn_name: on_spawn.clone(),
        });
    }

    if let Some(on_release) = &functions.on_released_fn {
        entity.insert(UiOnReleaseFnReq {
            fn_name: on_release.clone(),
        });
    }

    if tree.containers.is_empty() {
        return;
    }

    let parent = entity.id();
    let mut commands = entity.commands();
    for component in &tree.containers {
        let mut children = commands.spawn_empty();
        spawn_layout(&mut children, &component, server);
        children.insert(ChildOf(parent));
    }
}

#[derive(Component)]
pub(crate) struct UiTemplatePrepared;

pub(crate) fn spawn_template(
    mut commands: Commands,
    mut templates: Query<(Entity, &UiDocumentTemplate), Without<UiTemplatePrepared>>,
    mut query:     Query<(Entity, &UiId)>,
    assets:        Res<Assets<UiLayout>>,
    server:        Res<AssetServer>,
) {
    templates.iter_mut().for_each(|(tmp, template)| {
        if let Some(layout) = assets.get(&template.target_layout) {

            for (c, ui_id) in query.iter_mut() {
                if ui_id.0 != template.target_container {
                    continue;
                }
                let target_template = layout.templates.get(&template.name).unwrap();
                let mut tree = target_template.root.clone();

                set_properties(&mut tree, &template.properties);

                let mut c_commands = commands.entity(c);
                spawn_layout(&mut c_commands, &tree, &server);

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
pub struct UiQuery<'w, 's, T: 'static + Component> {
    query: Query<'w, 's, (&'static T, &'static UiId)>
}

impl<'w, 's, T: 'static + Component> UiQuery<'w, 's, T> {
    pub fn get(&self, id: &str) -> Option<&T> {
        self.query.iter()
            .find(|(_, ui_id)| ui_id.0 == id)
            .map(|(component, _)| component)
    }
}

#[derive(SystemParam)]
pub struct UiMutQuery<'w, 's, T: 'static + Component<Mutability = Mutable>> {
    query: Query<'w, 's, (&'static mut T, &'static UiId)>
}

impl<'w, 's, T: 'static + Component<Mutability=Mutable>> UiMutQuery<'w, 's, T> {
    pub fn get_mut(&mut self, id: &str) -> Option<Mut<T>> {
        for (c, ui_id) in self.query.iter_mut() {
            if ui_id.0 == id {
                return Some(c);
            }
        }

        None
    }
}

#[derive(SystemParam)]
pub struct UiFunctionRegistry<'w, 's> {
    functions: ResMut<'w, UiFunctions>,
    cmd: Commands<'w, 's>,
}

pub type SpawnFunction = dyn Fn(EntityCommands) + Send + Sync + 'static;

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
pub(crate) struct UiFunctions {
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

#[derive(Component)]
pub(crate) struct UiSpawnFnReq {
    fn_name: String,
}

#[derive(Component)]
pub(crate) struct UiOnReleaseFnReq {
    fn_name: String,
}

pub(crate) fn observe_on_spawn(
    mut cmd:           Commands,
    function_bindings: Res<UiFunctions>,
    on_spawn:          Query<(Entity, &UiSpawnFnReq), Added<UiSpawnFnReq>>,
) {
    on_spawn.iter().for_each(|(entity, req)| {
        function_bindings.maybe_run(&req.fn_name, entity, &mut cmd);
        cmd.entity(entity).remove::<UiSpawnFnReq>();
    });
}

/*pub(crate) fn observe_interaction(
    mut cmd: Commands,
    interactions: Query<(Entity, &AdvancedInteraction), Changed<AdvancedInteraction>>,
    functions: Res<UiFunctions>,
    on_release: Query<&UiOnReleaseFnReq>,
){
    interactions.iter().for_each(|(entity, interaction)|{
        match interaction {
            AdvancedInteraction::Released => {
                if let Ok(x) = on_release.get(entity) {
                    functions.maybe_run(&x.fn_name, entity, &mut cmd);
                }
            }
            _ => {}
        }
    });
}
*/