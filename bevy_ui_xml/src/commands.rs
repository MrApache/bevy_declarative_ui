use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bevy::asset::uuid;
use bevy::ecs::system::{SystemId, SystemParam, SystemState};
use bevy::prelude::*;
use uuid::Uuid;
use crate::loader::{ParsedTree, XmlAsset};
use crate::parser::Resources;
use crate::{Layouts, UiDocumentTemplate, XmlLibrary};
use crate::prelude::Extractor;
use crate::xml_component::XmlComponent;

#[derive(Reflect, Component, Clone)]
pub struct UiDocument {
    handle: Handle<XmlAsset>,
    resources: Resources,
    id: Uuid,
}

impl UiDocument {
    pub fn new(handle: Handle<XmlAsset>) -> Self {
        Self {
            handle,
            resources: Default::default(),
            id: Uuid::new_v4(),
        }
    }

    pub fn set_property(&mut self, name: &str, value: impl Into<String>) {
        if !self.resources.contains_key(name) {
            error!("[Ui Resources] Property '{}' not found", name);
            return;
        }

        self.resources.insert(name, value);
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Component)]
pub(crate) struct InjectorComponents {
    injectors: HashMap<String, Vec<(Arc<RwLock<Box<dyn XmlComponent>>>, String)>>,
    document: Uuid,
}

impl InjectorComponents {
    fn new(id: Uuid) -> Self {
        Self {
            injectors: HashMap::new(),
            document: id,
        }
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
    pub(crate) fn new(global: &'a Resources, local: &'a Resources) -> Self {
        Self { global, local }
    }

    pub(crate) fn get(&self, key: &str) -> Option<&String> {
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

pub(crate) fn asset_event_reader(
    mut commands: Commands,
    mut events:   EventReader<AssetEvent<XmlAsset>>,
    documents:    Query<(Entity, &UiDocument)>,
    children:     Query<&Children>,
    mut layouts:  ResMut<Layouts>,
    library:      Res<XmlLibrary>,
    server:       Res<AssetServer>,
    assets:       Res<Assets<XmlAsset>>,
) {
    events.read().for_each(|ev| {
        match ev {
            AssetEvent::Modified { id } => {
                parse_xml(*id, &assets, &library, &mut layouts);
                hot_reload(*id, &mut commands, &layouts, &library, &server, documents, children);
            }
            AssetEvent::Added { id } => {
                parse_xml(*id, &assets, &library, &mut layouts);
            }
            _ => return
        }
    });
}

fn parse_xml(
    id:      AssetId<XmlAsset>,
    assets:  &Assets<XmlAsset>,
    library: &XmlLibrary,
    layouts: &mut Layouts,
) {
    let asset: &XmlAsset = assets.get(id).unwrap();
    layouts.insert(id, crate::parser::parse_layout(&library, asset));
}

fn hot_reload(
    id:        AssetId<XmlAsset>,
    commands:  &mut Commands,
    layouts:   &Layouts,
    library:   &XmlLibrary,
    server:    &AssetServer,
    documents: Query<(Entity, &UiDocument)>,
    children:  Query<&Children>,
) {
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
            let resources = ResourceCollection::new(&layout.global, &local);
            spawn_layout(
                aa.id,
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
}

pub(crate) fn spawn_command(
    mut commands:  Commands,
    mut documents: Query<(Entity, &mut UiDocument), Without<UiDocumentPrepared>>,
    layouts:       Res<Layouts>,
    server:        Res<AssetServer>,
    library:       Res<XmlLibrary>,
) {
    for (e, mut document) in documents.iter_mut() {
        if layouts.contains_key(&document.handle.id()) {
            let layout = layouts.get(&document.handle.id()).unwrap();
            document.resources = layout.local.clone();

            let mut entity = commands.entity(e);
            let resources = ResourceCollection::new(&layout.global, &document.resources);
            spawn_layout(
                document.id,
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

#[derive(Component, Reflect)]
pub struct UiDocumentId {
    id: Uuid
}

impl UiDocumentId {
    pub fn id(&self) -> Uuid {
        self.id
    }
}

pub(crate) fn spawn_layout(
    id:        Uuid,
    entity:    &mut EntityCommands,
    server:    &AssetServer,
    library:   &XmlLibrary,
    tree:      &ParsedTree,
    resources: &ResourceCollection,
) {
    entity.insert(UiDocumentId {
        id
    });

    tree.components.iter().for_each(|(c, _)| {
        c.insert_to(entity, server);
    });

    tree.container_properties.iter().for_each(|(name, property)| {
        if name == "id" {
            entity.insert(UiId(resources.get(property).unwrap().clone()));
        }
        else if let Some(function) = library.functions.get(name.as_str()) {
            if let Some(resource) = resources.get(property) {
                function.insert_function_tag(resource, entity);
            }
            else {
                warn!("[Ui Resources] Resource '{}' not found", property)
            }
        }
        else {
            error!("[Container] Unknown attribute: {}", name);
        }
    });

    tree.components.iter().for_each(|(c, vec)| {
        if vec.is_empty() {
            return;
        }

        let mut injectors:InjectorComponents = InjectorComponents::new(id);
        let component = Arc::new(RwLock::new(dyn_clone::clone_box(&**c)));
        for ap in vec {
            if !injectors.injectors.contains_key(&ap.property) {
                injectors.injectors.insert(ap.property.clone(), Vec::new());
            }

            let vec = injectors.injectors.get_mut(&ap.property).unwrap();
            vec.push((component.clone(), ap.attribute.clone()));
        }

        entity.insert(injectors);
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
            id,
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
    mut commands:  Commands,
    mut templates: Query<(Entity, &UiDocumentTemplate), Without<UiTemplatePrepared>>,
    mut query:     Query<(Entity, &UiId, &UiDocumentId)>,
    //documents:     Query<&UiDocument>,

    layouts:       Res<Layouts>,
    library:       Res<XmlLibrary>,
    server:        Res<AssetServer>,
) {
    templates.iter_mut().for_each(|(tmp, template)| {
        if layouts.contains_key(&template.target_layout.id()) {
            for (c, ui_id, document_id) in query.iter_mut() {
                if ui_id.0 != template.target_container {
                    continue;
                }
                let layout = layouts.get(&template.target_layout.id()).unwrap();
                let target_template = layout.templates.get(&template.name).unwrap();

                let mut tree = target_template.root.clone();

                let local = &template.resources;
                //TODO Ui Document resources
                let resources = ResourceCollection::new(&layout.global, &local);

                set_component_properties(&mut tree, &resources);

                let mut c_commands = commands.entity(c);
                spawn_layout(
                    document_id.id,
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
    tree.components.iter_mut().for_each(|(c, vec)| {
        vec.iter().for_each(|ap| {
            let value = properties.get(&ap.property)
                .expect(&format!("[Ui Layout Template] Property not found: {}", ap.property));

            c.parse_attribute(&ap.attribute, value);
        })
    });

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

pub(crate) fn sync_local_resources(
    world:  &mut World,
    params: &mut SystemState<(
        Res<AssetServer>,
        Query<&UiDocument, Changed<UiDocument>>,
        Query<(Entity, &mut InjectorComponents)>
    )>,
) {
    let (server, documents, mut injectors) = params.get_mut(world);
    let server = server.clone();

    // Сначала собираем все задачи
    let mut pending_injections = Vec::new();

    for doc in documents.iter() {
        for (entity, mut injector) in injectors.iter_mut() {
            if doc.id != injector.document {
                continue;
            }

            for (name, value) in doc.resources.iter() {
                if let Some(vec) = injector.injectors.get_mut(name.as_str()) {
                    for (arc, attribute) in vec.iter_mut() {
                        // Сохраняем всё, что нужно для вызова
                        pending_injections.push((
                            entity,
                            arc.clone(),
                            attribute.clone(),
                            value.clone(),
                        ));
                    }
                }
            }
        }
    }

    // Потом выполняем инъекции, уже имея полный `&mut World`
    for (entity, arc, attribute, value) in pending_injections {
        let writer = arc.write().unwrap();
        let mut extractor = Extractor::new(world, entity);
        writer.inject_value(&attribute, &value, &mut extractor, &server);
    }
}

/*pub(crate) fn sync_local_resources(
    mut world:     &mut World,
    server:        Res<AssetServer>,
    documents:     Query<&UiDocument, Changed<UiDocument>>,
    mut injectors: Query<(Entity, &mut InjectorComponents)>
) {
    documents.iter().for_each(|doc| {
        //println!("[Ui Resources] Sync local resources in {}", doc.id);
        injectors.iter_mut().for_each(|(entity, mut injector)| {
            if doc.id != injector.document {
                return;
            }

            doc.resources.iter().for_each(|(name, value)| {
                if let Some(vec) = injector.injectors.get_mut(name.as_str()) {
                    vec.iter_mut().for_each(|(arc, attribute)| {
                        let mut writer = arc.write().unwrap();
                        writer.inject_value(&mut world, entity, attribute.as_str(), value.as_str());
                        writer.parse_attribute(attribute.as_str(), value.as_str());
                        writer.insert_to(&mut commands, &server);
                        //println!("[Ui Resources] Sync component {}", entity);
                    });
                }
            });
        })
    })
}
*/