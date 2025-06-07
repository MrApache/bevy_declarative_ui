use std::collections::HashMap;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use thiserror::Error;
use crate::xml_component::XmlComponent;

#[derive(Debug)]
pub(crate) struct ParsedTree {
    pub(crate) components: Vec<Box<dyn XmlComponent>>,
    pub(crate) properties: Vec<AttributeProperty>,

    pub(crate) containers: Vec<ParsedTree>,
    pub(crate) container_properties: HashMap<String, String>,

    pub(crate) functions:  HashMap<String, String>,
    pub(crate) id: Option<String>,
}

pub trait AttributeFunction: Sync + Send + 'static {
    fn insert_function_tag(&self, value: &str, entity: &mut EntityCommands);
}

impl Clone for ParsedTree {
    fn clone(&self) -> Self {
        Self {
            components: self.components.iter().map(|c| dyn_clone::clone_box(&**c)).collect::<Vec<_>>(),
            containers: self.containers.clone(),
            properties: self.properties.clone(),
            container_properties: self.container_properties.clone(),
            functions:  self.functions.clone(),
            id: self.id.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AttributeProperty {
    pub(crate) attribute: String,
    pub(crate) property: String,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum UiXmlLoaderError {
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub(crate) struct UiTemplate {
    pub(crate) root: ParsedTree,
}

#[derive(Asset, TypePath)]
pub struct XmlAsset{
    pub(crate) string: String,
}

#[derive(Default)]
pub struct UiXmlLoader;

impl AssetLoader for UiXmlLoader {
    type Asset = XmlAsset;
    type Settings = ();
    type Error = UiXmlLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,

    ) -> Result<Self::Asset, Self::Error>
    {
        let mut string:String = String::new();
        reader.read_to_string(&mut string).await?;
        Ok(XmlAsset {
            string,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
    }
}