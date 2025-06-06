use std::marker::PhantomData;
use bevy::asset::*;
use bevy::prelude::*;

#[derive(Debug)]
pub struct RawHandle<A: Asset> {
    pub file: Option<String>,
    _marker: PhantomData<A>
}

impl<A: Asset> Clone for RawHandle<A> {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            ..Self::default()
        }
    }
}

impl<A: Asset> Default for RawHandle<A> {
    fn default() -> Self {
        Self {
            file: None,
            _marker: Default::default(),
        }
    }
}

impl<A: Asset> RawHandle<A> {
    pub fn new(file: String) -> Self {
        Self {
            file: Some(file),
            _marker: Default::default(),
        }
    }

    pub fn handle(&self, server: &AssetServer) -> Handle<A> {
        if let Some(file) = self.file.as_ref() {
            server.load::<A>(file)
        }
        else {
            Handle::default()
        }
    }

    pub fn new_opt(file: Option<String>) -> Self {
        Self {
            file,
            _marker: Default::default(),
        }
    }
}