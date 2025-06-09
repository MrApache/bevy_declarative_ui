use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bevy::asset::uuid;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use uuid::Uuid;
use crate::prelude::Extractor;
use crate::xml_component::XmlComponent;
use crate::xml_parser::{XmlLayout, Resources};
use crate::{Layouts, UiDocumentTemplate, XmlLibrary};
use crate::parser::{CompiledLayout, CompiledNode, LayoutCompiler};

#[derive(Resource, Reflect, Default)]
pub(crate) struct GlobalResources {
    storage: HashMap<AssetId<XmlLayout>, Resources>
}

#[derive(Reflect, Component, Clone)]
pub struct UiDocument {
    layout_id:    Handle<XmlLayout>,
    resources: Resources,
    id: Uuid,
}

impl UiDocument {
    pub fn new(handle: Handle<XmlLayout>) -> Self {
        Self {
            layout_id: handle,
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
pub(crate) struct Injector {
    injectors: HashMap<String, Vec<(Arc<RwLock<Box<dyn XmlComponent>>>, String)>>,
    layout_id: Handle<XmlLayout>,
    document_id: Uuid,
}

impl Injector {
    fn new(document_id: Uuid, layout_id: Handle<XmlLayout>) -> Self {
        Self {
            injectors: HashMap::new(),
            layout_id,
            document_id,
        }
    }
}

#[derive(Component)]
pub(crate) struct UiDocumentPrepared;

#[derive(Component, Reflect, Debug, Clone, Default)]
pub(crate) struct UiContainerId(pub String);

#[derive(Component, Reflect)]
pub struct UiDocumentId {
    id: Uuid
}

impl UiDocumentId {
    pub fn id(&self) -> Uuid {
        self.id
    }
}

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
    mut events:   EventReader<AssetEvent<XmlLayout>>,
    documents:    Query<(Entity, &UiDocument)>,
    children:     Query<&Children>,
    mut g_res:    ResMut<GlobalResources>,
    mut layouts:  ResMut<Layouts>,
    library:      Res<XmlLibrary>,
    server:       Res<AssetServer>,
    assets:       Res<Assets<XmlLayout>>,
) {
    events.read().for_each(|ev| {
        match ev {
            AssetEvent::Modified { id } => {
                parse_xml(&mut g_res, *id, &assets, &library, &mut layouts);
                hot_reload(*id, &mut commands, &layouts, &library, &server, documents, children);
            }
            AssetEvent::Added { id } => {
                parse_xml(&mut g_res, *id, &assets, &library, &mut layouts);
            }
            _ => return
        }
    });
}

fn parse_xml(
    g_res:   &mut GlobalResources,
    id:      AssetId<XmlLayout>,
    assets:  &Assets<XmlLayout>,
    library: &XmlLibrary,
    layouts: &mut Layouts,
) {
    let layout: &XmlLayout = assets.get(id).unwrap();
    let compiled_layout: CompiledLayout = LayoutCompiler::new(library, layout).compile();

    g_res.storage.insert(id, layout.global.clone()); //TODO Fix
    //println!("Compile: {}", id);
    layouts.insert(id, compiled_layout);
}

fn hot_reload(
    id:        AssetId<XmlLayout>,
    commands:  &mut Commands,
    layouts:   &Layouts,
    library:   &XmlLibrary,
    server:    &AssetServer,
    documents: Query<(Entity, &UiDocument)>,
    children:  Query<&Children>,
) {
    for (e, aa) in documents {
        if !id.eq(&aa.layout_id.id()) {
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

        if layouts.contains_key(&aa.layout_id.id()) {
            let mut entity: EntityCommands = commands.entity(e);
            let layout = layouts.get(&aa.layout_id.id()).unwrap();
            let local = Resources::default();
            let resources = ResourceCollection::new(&layout.global, &local);
            spawn_layout(
                aa.layout_id.clone(),
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
    mut g_res:     ResMut<GlobalResources>,
    mut commands:  Commands,
    mut documents: Query<(Entity, &mut UiDocument), Without<UiDocumentPrepared>>,
    layouts:       Res<Layouts>,
    server:        Res<AssetServer>,
    library:       Res<XmlLibrary>,
) {
    for (e, mut document) in documents.iter_mut() {
        if layouts.contains_key(&document.layout_id.id()) {
            let layout = layouts.get(&document.layout_id.id()).unwrap();
            document.resources = layout.local.clone();

            g_res.storage.insert(document.layout_id.id(), layout.global.clone()); //TODO Fix
            let mut entity = commands.entity(e);
            let resources = ResourceCollection::new(&layout.global, &document.resources);
            spawn_layout(
                document.layout_id.clone(),
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

pub(crate) fn spawn_layout(
    layout_id: Handle<XmlLayout>,
    doc_id:    Uuid,
    entity:    &mut EntityCommands,
    server:    &AssetServer,
    library:   &XmlLibrary,
    tree:      &CompiledNode,
    resources: &ResourceCollection,
) {
    entity.insert(UiDocumentId {
        id: doc_id,
    });

    tree.components.iter().for_each(|(c, _)| {
        c.insert_to(entity, server);
    });

    tree.properties.iter().for_each(|(name, property)| {
        if name == "id" {
            entity.insert(UiContainerId(resources.get(property).unwrap().clone()));
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

        let mut injectors: Injector = Injector::new(doc_id, layout_id.clone());
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
        entity.insert(UiContainerId(id.clone()));
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
            layout_id.clone(),
            doc_id,
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
    mut query:     Query<(Entity, &UiContainerId, &UiDocumentId)>,
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
                let mut tree = layout.templates.get(&template.name).unwrap().clone();

                //TODO Ui Document resources
                let local = &template.resources;
                let resources = ResourceCollection::new(&layout.global, &local);

                set_component_properties(&mut tree, &resources);

                let mut c_commands = commands.entity(c);
                spawn_layout(
                    Default::default(), //TODO
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

fn set_component_properties(tree: &mut CompiledNode, properties: &ResourceCollection) {
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

type PendingInjections = Vec<(Entity, Arc<RwLock<Box<dyn XmlComponent>>>, String, String)>;

pub(crate) fn sync_local_resources(
    world:  &mut World,
    params: &mut SystemState<(
        Res<GlobalResources>,
        Res<AssetServer>,
        Query<&UiDocument, Changed<UiDocument>>,
        Query<(Entity, &mut Injector)>,
        Local<PendingInjections>,
    )>,
) {
    let (g_res,
        server,
        documents,
        mut injectors,
        mut pending_injections) = params.get_mut(world);
    let server = server.clone();

    fn push(
        name:     &str,
        value:    &str,
        entity:   Entity,
        injector: &mut Injector,
        pending_injections: &mut PendingInjections,
    ) {
        if let Some(vec) = injector.injectors.get_mut(name) {
            for (arc, attribute) in vec.iter_mut() {
                pending_injections.push((
                    entity,
                    arc.clone(),
                    attribute.clone(),
                    value.to_string(),
                ));
            }
        }
    }

    if g_res.is_changed() || g_res.is_added() {
        for (entity, mut injector) in injectors.iter_mut() {
            let global_resources = g_res.storage.get(&injector.layout_id.id());
            if let Some(global_resources) = global_resources {
                for (name, value) in global_resources.iter() {
                    push(name, value, entity, &mut injector, &mut *pending_injections);
                }
            }
        }
    }

    for doc in documents.iter() {
        for (entity, mut injector) in injectors.iter_mut() {
            if doc.id != injector.document_id {
                continue;
            }

            for (name, value) in doc.resources.iter() {
                push(name, value, entity, &mut injector, &mut *pending_injections);
            }
        }
    }

    pending_injections.retain(|(entity, arc, attribute, value)| {
        let writer = arc.write().unwrap();
        let mut extractor = Extractor::new(world, *entity);
        writer.inject_value(&attribute, &value, &mut extractor, &server);
        println!("Inject value to entity: {}", entity);
        false
    });
}