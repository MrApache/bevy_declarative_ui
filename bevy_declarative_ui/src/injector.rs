use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use bevy::asset::AssetServer;
use bevy::prelude::Component;
use crate::prelude::{Extractor, ValueStorage};

pub trait Injector: Send + Sync + 'static {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, server: &AssetServer);
}

#[derive(Component, Default)]
pub(crate) struct ValueInjectors {
    //1 - Property itself, 2 - Component, 3 - name of attribute
    pub injectors: HashMap<TypeId, Vec<(Arc<Box<dyn Injector>>, String)>>,
}