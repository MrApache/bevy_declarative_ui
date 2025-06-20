use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use crate::prelude::{CallbacksArguments, CallbackInjector, Callbacks, Extractor, GlobalResources, UiResources, ValueStorage};
use crate::xml_parser::XmlLayout;
use crate::{Layouts, XmlLibrary};
use crate::injector::{Injector, ValueInjectors};
use crate::parser::{CompiledLayout, CompiledNode, FunctionType, LayoutCompiler};
use crate::resources::Storage;
use crate::templates::*;

#[derive(Component)]
pub struct RootDocument;

#[derive(Component, Reflect, Clone)]
pub struct UiContext {
    ///Id of XmlLayout asset
    layout_handle: Handle<XmlLayout>,
    ///Id of Main Root
    root_entity:  Entity,
    ///Id of Sub Root
    subtree_root: Entity,
    ///Id of Current Root
    owner_entity: Entity,
}

impl UiContext {
    ///Id of XmlLayout asset
    pub fn layout_handle(&self) -> Handle<XmlLayout> {
        self.layout_handle.clone()
    }

    ///Id of Main Root
    pub const fn root_entity(&self) -> Entity {
        self.root_entity
    }

    ///Id of Sub Root
    pub const fn subtree_root(&self) -> Entity {
        self.subtree_root
    }

    ///Id of Current Root
    pub const fn owner_entity(&self) -> Entity {
        self.owner_entity
    }
}

#[derive(Component, Default)]
pub struct Containers {
    map: HashMap<String, Entity>,
}

#[derive(Bundle)]
pub struct UiDocumentBundle {
    root:       RootDocument,
    context:    UiContext,
    requests:   Templates,
    containers: Containers,
    resources:  UiResources,
}

impl UiDocumentBundle {
    pub fn new(handle: Handle<XmlLayout>) -> Self {
        Self {
            root:    RootDocument,
            context: UiContext {
                layout_handle: handle,
                root_entity:   Entity::PLACEHOLDER,
                subtree_root:  Entity::PLACEHOLDER,
                owner_entity:  Entity::PLACEHOLDER,
            },
            requests:   Templates::default(),
            containers: Containers::default(),
            resources:  UiResources::default(),
        }
    }
}

#[derive(Component)]
pub struct UiDocumentPrepared;

#[derive(Component, Reflect, Debug, Clone, Default)]
pub(crate) struct UiContainerId(pub String);

pub(crate) fn asset_event_reader(
    mut commands: Commands,
    mut events:   EventReader<AssetEvent<XmlLayout>>,
    mut docs:     Query<(
        Entity,
        &mut UiResources,
        &mut Containers,
        &mut Templates,
        &UiContext,
    )>,
    mut g_res:    ResMut<GlobalResources>,
    mut layouts:  ResMut<Layouts>,
    mut assets:   ResMut<Assets<XmlLayout>>,
    library:      Res<XmlLibrary>,
    server:       Res<AssetServer>,
) {
    events.read().for_each(|ev| {
        match ev {
            AssetEvent::Modified { id } => {
                parse_xml(*id, &mut g_res, &mut assets, &library, &mut layouts);
                hot_reload(*id, &mut commands, &mut layouts, &library, &server, &mut docs);
            }
            AssetEvent::Added { id } => {
                parse_xml(*id, &mut g_res, &mut assets, &library, &mut layouts);
            }
            _ => return
        }
    });
}

fn parse_xml(
    id:      AssetId<XmlLayout>,
    g_res:   &mut GlobalResources,
    assets:  &Assets<XmlLayout>,
    library: &XmlLibrary,
    layouts: &mut Layouts,
) {
    let layout: &XmlLayout = assets.get(id).unwrap();
    let mut compiled_layout: CompiledLayout = LayoutCompiler::new(library, layout).compile();
    g_res.storage.insert(id, std::mem::take(&mut compiled_layout.global));
    layouts.insert(id, compiled_layout);
}

fn hot_reload(
    id:        AssetId<XmlLayout>,
    commands:  &mut Commands,
    layouts:   &mut Layouts,
    library:   &XmlLibrary,
    server:    &AssetServer,
    documents: &mut Query<(
        Entity,
        &mut UiResources,
        &mut Containers,
        &mut Templates,
        &UiContext,
    )>,
) {
    for (e, mut resources, mut containers, mut requests, context) in documents {
        if !id.eq(&context.layout_handle.id()) {
            continue;
        }

        commands.entity(e).despawn_related::<Children>();
        requests.queue.clear();
        requests.spawned.clear();
        containers.map.clear();

        if layouts.contains_key(&context.layout_handle.id()) {
            let mut entity: EntityCommands = commands.entity(e);
            let layout: &mut CompiledLayout = layouts.get_mut(&context.layout_handle.id()).unwrap();
            resources.properties = std::mem::take(&mut layout.local.properties);
            spawn_layout(
                &context,
                &mut containers,
                &mut entity,
                &server,
                &library,
                &layout.root,
                &layout.types,
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
    mut documents: Query<(
        Entity,
        &mut UiResources,
        &mut Containers,
        &mut UiContext
    ), Without<UiDocumentPrepared>>,
    mut layouts:   ResMut<Layouts>,
    server:        Res<AssetServer>,
    library:       Res<XmlLibrary>,
) {
    for (e, mut resources, mut containers, mut context) in documents.iter_mut() {
        if layouts.contains_key(&context.layout_handle.id()) {
            let layout: &mut CompiledLayout = layouts.get_mut(&context.layout_handle.id()).unwrap();
            resources.properties = std::mem::take(&mut layout.local.properties);
            context.root_entity = e;
            context.subtree_root = e;
            context.owner_entity = e;
            g_res.changed = true;

            let mut entity = commands.entity(e);

            spawn_layout(
                &context,
                &mut containers,
                &mut entity,
                &server,
                &library,
                &layout.root,
                &layout.types,
            );
            commands.entity(e).insert(UiDocumentPrepared);
        }
    }
}

pub(crate) fn spawn_layout(
    context:    &UiContext,
    containers: &mut Containers,
    entity:     &mut EntityCommands,
    server:     &AssetServer,
    library:    &XmlLibrary,
    tree:       &CompiledNode,
    types:      &HashMap<String, TypeId>,
) {
    insert_components(&context, containers, entity, server, library, tree, types);

    if tree.containers.is_empty() {
        return;
    }

    let parent = entity.id();
    let mut commands = entity.commands();
    for container in &tree.containers {
        let mut children = commands.spawn_empty();
        let mut context = context.clone();
        context.owner_entity = children.id();
        spawn_layout(
            &context,
            containers,
            &mut children,
            &server,
            &library,
            &container,
            &types,
        );
        children.insert(ChildOf(parent));
    }
}

fn insert_components(
    context:    &UiContext,
    containers: &mut Containers,
    entity:     &mut EntityCommands,
    server:     &AssetServer,
    library:    &XmlLibrary,
    tree:       &CompiledNode,
    types:      &HashMap<String, TypeId>,
) {
    let mut injectors: ValueInjectors = ValueInjectors::default();

    tree.components.iter().for_each(|component| {
        component.value.insert_to(entity, server);

        if component.properties.is_empty() {
            return;
        }

        let arc_component: Arc<Box<dyn Injector>> = Arc::new(component.value.as_injector());
        for ap in &component.properties {
            let type_id: &TypeId = types.get(&ap.property)
                .expect(&format!("Type Id for property '{}' not found", ap.property));
            if !injectors.injectors.contains_key(type_id) {
                injectors.injectors.insert(*type_id, Vec::new());
            }

            let vec = injectors.injectors.get_mut(type_id).unwrap();
            vec.push((arc_component.clone(), ap.attribute.clone()));
        }
    });

    if let Some(id) = &tree.id {
        entity.insert(UiContainerId(id.clone()));
        containers.map.insert(id.clone(), entity.id());
    }

    let mut callbacks: Callbacks = Callbacks::default();
    let mut arguments: CallbacksArguments = CallbacksArguments::default();
    for (name, function) in &tree.functions {
        let callback = library.functions.get(name.as_str()).unwrap();
        match &function.kind {
            FunctionType::Value => {
                callback.insert_callback(entity);
                callbacks.insert(callback.type_id(), function.value.to_string());
            }
            FunctionType::Property(property) => {
                let type_id: &TypeId = types.get(property)
                    .expect(&format!("Type Id for property '{}' not found", property));

                if !injectors.injectors.contains_key(type_id) {
                    injectors.injectors.insert(*type_id, Vec::new());
                }

                let injector: CallbackInjector = CallbackInjector::new(callback.type_id());
                let boxed_injector: Box<dyn Injector> = Box::new(injector);

                injectors.injectors.get_mut(type_id)
                    .unwrap()
                    .push((Arc::new(boxed_injector), name.clone()));

                callback.insert_callback(entity);
                callbacks.insert(callback.type_id(), function.value.clone());
            },
            FunctionType::CallFunction(args) => {
                callback.insert_callback(entity);
                callbacks.insert(callback.type_id(), function.value.to_string());
                if !args.is_empty() {
                    arguments.insert(callback.type_id(), args.clone());
                }
            }
        }
    }

    if !injectors.injectors.is_empty() || !callbacks.is_empty() || tree.id.is_some() {
        entity.insert(context.clone());
        if !callbacks.is_empty() {
            entity.insert(callbacks);
            if !arguments.is_empty() {
                entity.insert(arguments);
            }
        }
    }

    if !injectors.injectors.is_empty() {
        entity.insert(injectors);
    }
}

fn spawn_template_layout(
    mut bundle: UiTemplateBundle,
    containers: &mut Containers,
    entity:     &mut EntityCommands,
    server:     &AssetServer,
    library:    &XmlLibrary,
    tree:       &CompiledNode,
    types:      &HashMap<String, TypeId>,
) -> Entity {
    if tree.containers.is_empty() {
        insert_components(&bundle.context, containers, entity, server, library, tree, types);
        return bundle.context.subtree_root;
    };

    let parent = entity.id();
    let mut commands = entity.commands();
    let mut children = commands.spawn_empty();

    bundle.context.subtree_root = children.id();
    bundle.context.owner_entity = children.id();
    let id: UiContext = bundle.context.clone();

    children.insert(ChildOf(parent));
    children.insert(UiDocumentPrepared);
    children.insert(bundle);
    insert_components(&id, containers, &mut children, server, library, tree, types);

    let mut iter = tree.containers.iter();
    spawn_layout(
        &id,
        containers,
        &mut children,
        &server,
        &library,
        &iter.next().unwrap(),
        &types,
    );

    for container in iter {
        let mut children = commands.spawn_empty();
        spawn_layout(
            &id,
            containers,
            &mut children,
            &server,
            &library,
            &container,
            &types,
        );
        children.insert(ChildOf(parent));
    }

    id.subtree_root
}

pub(crate) fn template_actions(
    mut commands:  Commands,
    mut documents: Query<(&mut Templates, &Containers, &UiContext), Changed<Templates>>,
    mut s_tmpl:    Query<&mut UiResources, With<Template>>,
    mut layouts:   ResMut<Layouts>,
    library:       Res<XmlLibrary>,
    server:        Res<AssetServer>,
    g_res:         Res<GlobalResources>,
) {
    documents.iter_mut().for_each(|(mut templates, containers, context)| {
        if templates.queue.is_empty() {
            return;
        }

        let spawn_requests = std::mem::take(&mut templates.queue);
        templates.queue.clear();

        spawn_requests.into_iter().for_each(|mut request| {
            match request.action {
                TemplateAction::Spawn => {
                    let name = std::mem::take(&mut request.instance_name);
                    let root_entity = spawn_template(
                        request,
                        containers,
                        context,
                        &mut commands,
                        &mut layouts,
                        &library,
                        &server,
                        &g_res
                    );
                    templates.spawned.insert(name, root_entity);
                }
                TemplateAction::SpawnOrInsert => {
                    if templates.spawned.contains_key(&request.instance_name) {
                        insert_template_resources(&mut request, &templates, &mut s_tmpl);
                    } else {
                        let name = std::mem::take(&mut request.instance_name);
                        let root_entity = spawn_template(
                            request,
                            containers,
                            context,
                            &mut commands,
                            &mut layouts,
                            &library,
                            &server,
                            &g_res
                        );
                        templates.spawned.insert(name, root_entity);
                    }
                }
                TemplateAction::Insert => insert_template_resources(&mut request, &templates, &mut s_tmpl),
            }
        });
    })
}

fn insert_template_resources(request: &mut TemplateRequest, templates: &Templates, s_tmpl: &mut Query<&mut UiResources, With<Template>>) {
    let entity = templates.spawned.get(&request.instance_name).unwrap();
    let buffer = request.resources.take_changed();

    let mut resources = s_tmpl.get_mut(*entity).unwrap();
    buffer.into_iter().for_each(|(id, mut storage)| {
        storage.storage.set_changed();
        resources.add_property_internal(id, storage);
    });
}

fn spawn_template(
    request:    TemplateRequest,
    containers: &Containers,
    context:    &UiContext,

    commands:   &mut Commands,
    layouts:    &mut Layouts,
    library:    &XmlLibrary,
    server:     &AssetServer,
    g_res:      &GlobalResources,
) -> Entity {
    let layout: &mut CompiledLayout = layouts.0.get_mut(&context.layout_handle.id()).unwrap();
    let template = layout.templates.get_mut(&request.name)
        .expect(&format!("Template '{}' not found", request.name));

    write_default_values_in_template(
        &mut template.root,
        &request.resources,
        &g_res.storage.get(&context.layout_handle.id()).unwrap(),
        &layout.types
    );

    if !template.allowed_containers.contains(&request.container) {
        panic!("TOdo error");
    }

    let container = containers.map.get(&request.container)
        .expect(&format!("Container with id '{}' in entity '{}' not found", request.container, context.root_entity));

    let mut entity = commands.entity(*container);
    let template_bundle = UiTemplateBundle {
        context: UiContext {
            layout_handle: context.layout_handle.clone(),
            root_entity: context.root_entity,
            subtree_root: entity.id(),
            owner_entity: entity.id(),
        },
        resources: request.resources,
        tag: Template,
    };

    let mut containers = Containers::default();
    spawn_template_layout(
        template_bundle,
        &mut containers,
        &mut entity,
        &server,
        &library,
        &template.root,
        &layout.types
    )
}

fn write_default_values_in_template(
    tree:   &mut CompiledNode,
    local:  &UiResources,
    global: &UiResources,
    types:  &HashMap<String, TypeId>
) {
    tree.components.iter_mut().for_each(|component| {
        component.properties.iter().for_each(|ap| {
            let type_id: &TypeId = types.get(&ap.property)
                .expect(&format!("Type Id for property '{}' not found", ap.property));

            let storage = if let Some(local) = local.get_property(*type_id) {
                local
            }
            else if let Some(global) = global.get_property(*type_id) {
                global
            }
            else {
                panic!("Type Id for property '{}' not found", ap.property);
            };

            let storage = ValueStorage::new(&storage.storage);
            component.value.write_value(&ap.attribute, &storage);
        })
    });

    tree.functions.iter_mut().for_each(|(_, function)| {
        if let FunctionType::Property(property) = &function.kind {
            let type_id: &TypeId = types.get(property)
                .expect(&format!("Type Id for property '{}' not found", property));

            let storage = if let Some(local) = local.get_property(*type_id) {
                local
            }
            else if let Some(global) = global.get_property(*type_id) {
                global
            }
            else {
                panic!("Type Id for property '{}' not found", property);
            };
            let storage: ValueStorage<'_> = ValueStorage::new(&storage.storage);
            //println!("Insert value to {} with {}", name, storage.read::<String>());
            function.value = storage.read::<String>().clone();
        }
    });

    for container in &mut tree.containers {
        write_default_values_in_template(container, local, global, types);
    }
}

struct Resource {
    type_id: TypeId,
    storage: Arc<Storage>,
}

type PendingInjections = Vec<PendingInjection>;

pub(crate) struct PendingInjection {
    entity:    Entity,
    attribute: String,
    injector:  Arc<Box<dyn Injector>>,
    storage:   Arc<Storage>
}

pub(crate) fn sync_resources(
    world: &mut World,
    params: &mut SystemState<(
        ResMut<GlobalResources>,
        Res<AssetServer>,
        Query<(&mut UiResources, &UiContext), (Changed<UiResources>, With<UiDocumentPrepared>)>,
        Query<(&mut ValueInjectors, &UiContext)>,
        Local<PendingInjections>,
    )>,
) {
    let mut changed_resources = HashMap::<Entity, Vec<Resource>>::new();
    let mut changed_global_resources = HashMap::<AssetId<XmlLayout>, Vec<Resource>>::new();

    {
        let (mut g_res,
            server,
            mut documents,
            mut injectors,
            mut queue
        ) = params.get_mut(world);

        let server = server.clone();

        if g_res.is_changed() {
            let g_res = g_res.bypass_change_detection();

            g_res.storage.iter_mut().for_each(|(layout_id, resources)| {
                //println!("Global resources with id: {:?}, prepared: {}", layout_id, buffer.len());
                let mut vec = vec![];
                for (id, storage) in resources.take_changed() {
                    vec.push(Resource {
                        type_id: id,
                        storage: Arc::new(storage),
                    });
                }
                if !vec.is_empty() {
                    changed_global_resources.insert(*layout_id, vec);
                }
            });
        }

        for (mut resources, context) in documents.iter_mut() {
            if context.subtree_root == Entity::PLACEHOLDER {
                continue;
            }

            let buffer = resources.take_changed();
            let mut vec = vec![];

            buffer.into_iter().for_each(|(type_id, storage)| {
                //println!("Take value storage from: {}", root_id.id);
                vec.push(Resource {
                    type_id,
                    storage: Arc::new(storage),
                });
            });

            if !vec.is_empty() {
                changed_resources.insert(context.subtree_root, vec);
            }
        }

        if changed_resources.is_empty() && changed_global_resources.is_empty() {
            return;
        }

        for (mut injector, context) in injectors.iter_mut() {
            if let Some(resources) = changed_resources.get_mut(&context.subtree_root) {
                resources.iter().for_each(|resource| {
                    push(resource.type_id, resource.storage.clone(), context.owner_entity, &mut injector, &mut *queue);
                });
            }
            if let Some(resources) = changed_global_resources.get_mut(&context.layout_handle.id()) {
                resources.iter().for_each(|resource| {
                    push(resource.type_id, resource.storage.clone(), context.owner_entity, &mut injector, &mut *queue);
                });
            }
        }

        queue.iter_mut().for_each(|i| {
            let value: ValueStorage = ValueStorage::new(&i.storage.storage);
            let mut extractor: Extractor = Extractor::new(world, i.entity);
            i.injector.inject_value(&i.attribute, &value, &mut extractor, &server);
            //println!("Inject value to entity: {}", entity);
        });

        queue.clear();
    }

    changed_resources.into_iter().for_each(|(e, resources)| {
        let mut entity = world.entity_mut(e);
        let mut en_resources = entity.get_mut::<UiResources>().unwrap();

        resources.into_iter().for_each(|resource| {
            //println!("Return local resource to entity: {}", e);
            let storage: Storage = Arc::try_unwrap(resource.storage).ok().unwrap();
            en_resources.add_property_internal(resource.type_id, storage);
        })
    });

    let mut global_resources = world.resource_mut::<GlobalResources>();
    let global_resources = global_resources.bypass_change_detection();
    changed_global_resources.into_iter().for_each(|(id, resources)| {
        let global_resources = global_resources.storage.get_mut(&id).unwrap();

        resources.into_iter().for_each(|resource| {
            let storage: Storage = Arc::try_unwrap(resource.storage).ok().unwrap();
            global_resources.add_property_internal(resource.type_id, storage);
            //println!("Return global resource: {}", id);
        });
    })
}

fn push(
    name:     TypeId,
    value:    Arc<Storage>,
    entity:   Entity,
    injector: &mut ValueInjectors,
    queue:    &mut PendingInjections,
) {
    if let Some(vec) = injector.injectors.get_mut(&name) {
        for (arc, attribute) in vec.iter_mut() {
            queue.push(PendingInjection {
                entity,
                attribute: attribute.clone(),
                injector: arc.clone(),
                storage: value.clone(),
            });
        }
    }
}