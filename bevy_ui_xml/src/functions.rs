use std::collections::HashMap;
use bevy::ecs::system::{SystemId, SystemParam};
use bevy::log::error;
use bevy::prelude::{Commands, Entity, In, IntoSystem, ResMut, Resource};

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
