use std::any::{Any, TypeId};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::ecs::system::{SystemId, SystemParam};
use crate::commands::UiContext;
use crate::prelude::*;

pub struct CallbackContext {
    ///Id of XmlLayout asset
    layout_handle: AssetId<XmlLayout>,
    ///Id of Main Root
    root_entity:  Entity,
    ///Id of Sub Root
    subtree_root: Entity,
    ///Id of Current Root
    owner_entity: Entity,
    ///Function component
    caller: TypeId,
}

impl CallbackContext {
    pub const fn layout_handle(&self) -> AssetId<XmlLayout> {
        self.layout_handle
    }

    pub const fn root_entity(&self) -> Entity {
        self.root_entity
    }

    pub const fn subtree_root(&self) -> Entity {
        self.subtree_root
    }

    pub const fn owner_entity(&self) -> Entity {
        self.owner_entity
    }

    pub const fn caller(&self) -> TypeId {
        self.caller
    }
}

#[derive(Reflect, Default, Component, Deref, DerefMut)]
pub struct CallbacksArguments(pub(crate) HashMap<TypeId, String>);

impl CallbacksArguments {
    pub fn arguments(&self, caller: TypeId) -> &str {
        self.get(&caller).unwrap()
    }
}

#[derive(Reflect, Default, Component, Deref, DerefMut)]
pub struct Callbacks(pub(crate) HashMap<TypeId, String>);

#[derive(SystemParam)]
pub struct UiCallbackInvoker<'w, 's> {
    callbacks: Query<'w, 's, &'static Callbacks>,
    functions: ResMut<'w, UiFunctions>,
    cmd: Commands<'w, 's>,
}

impl<'w, 's> UiCallbackInvoker<'w, 's> {

    pub fn try_run<C: AttributeCallback>(&mut self, context: &UiContext) {
        let cb_context: CallbackContext = CallbackContext {
            layout_handle: context.layout_handle().id(),
            root_entity: context.root_entity(),
            subtree_root: context.subtree_root(),
            owner_entity: context.owner_entity(),
            caller: TypeId::of::<C>(),
        };

        let callbacks: &Callbacks = self.callbacks.get(context.owner_entity()).unwrap();
        let function: &String = callbacks.get(&cb_context.caller).unwrap();

        if let Some(id) = self.functions.callbacks.get(function) {
            self.cmd.run_system_with(*id, cb_context);
        } else {
            error!("[Ui Functions] Function `{function}` is not bound on entity: {}", context.owner_entity());
        }
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
        S: IntoSystem<In<CallbackContext>, (), M> + 'static,
    {
        let id = self.cmd.register_system(func);
        let name = &name.into();
        self.functions.register(name, id);
    }

    pub fn register_event_handler<S, M, A>(&mut self, name: impl Into<String>, func: S) 
    where
        A: Send + Sync + 'static,
        S: IntoSystem<In<A>, (), M> + 'static,
    {
        let id = self.cmd.register_system(func);
        self.functions.register_event_handler(name, id);
    }
}

#[derive(Resource, Default)]
pub struct UiFunctions {
    callbacks: HashMap<String, SystemId<In<CallbackContext>>>,
    events:    HashMap<String, Box<dyn EventHandler>>,
}

impl UiFunctions {
    fn register(&mut self, key: impl Into<String>, system_id: SystemId<In<CallbackContext>>) {
        let key: String = key.into();
        self.callbacks.insert(key, system_id);
    }

    fn register_event_handler(&mut self, name: impl Into<String>, handler: impl EventHandler + 'static) {
        let name: String = name.into();
        self.events.insert(name, Box::new(handler));
    }

    pub fn get_event_handler(&self, name: &str) -> Option<&Box<dyn EventHandler>> {
        self.events.get(name)
    }
}

pub trait EventHandler: Send + Sync + Any + 'static {
     fn as_any(&self) -> &dyn Any;
}

impl<I, O: 'static> EventHandler for SystemId<I, O> where I: SystemInput + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait AttributeCallback: Send + Sync + 'static {
    fn insert_callback(&self, entity: &mut EntityCommands);
    fn type_id(&self) -> TypeId;
}

#[derive(Debug, Clone)]
pub(crate) struct AttributeProperty {
    pub(crate) attribute: String,
    pub(crate) property: String,
}