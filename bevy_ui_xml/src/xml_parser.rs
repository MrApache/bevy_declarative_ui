use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy_ui_xml_parser::{LayoutReader, PropertyValue, Resources, Template, UiNode, XmlLayoutError};

#[derive(Default, Debug)]
pub(crate) struct LayoutPath {
    pub current: String,
    pub global: String,
}

#[derive(Asset, TypePath, Default, Debug)]
pub struct XmlLayout {
    pub(crate) path:       LayoutPath,
    pub(crate) local:      Resources,
    pub(crate) global:     Resources,
    pub(crate) templates:  Vec<Template>,
    pub(crate) root_nodes: Vec<UiNode>,
}

impl XmlLayout {
    pub(crate) fn get_resource(&self, key: &str) -> Option<&PropertyValue> {
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


#[derive(Default)]
pub struct XmlLoader;

impl AssetLoader for XmlLoader {
    type Asset = XmlLayout;
    type Settings = ();
    type Error = XmlLayoutError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _: &(),
        ctx: &mut LoadContext<'_>,

    ) -> Result<Self::Asset, Self::Error>
    {
        let mut string:String = String::new();
        reader.read_to_string(&mut string).await?;

        let path = ctx.path().display().to_string();
        let mut reader = LayoutReader::new(&string, path.as_str());
        let result = reader.parse_layout();
        if let Ok(layout) = result {
            Ok(XmlLayout {
                path:       LayoutPath {
                    current: normalize_path(&ctx.path().display().to_string()),
                    global:  normalize_path(&ctx.path().display().to_string()),
                },
                local:      layout.local,
                global:     layout.global,
                templates:  layout.templates,
                root_nodes: layout.root_nodes,
            })
        }
        else {
            Err(result.err().unwrap())
        }
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
    }
}
fn normalize_path(path: &str) -> String {
    let without_xml = path.strip_suffix(".xml").unwrap_or(path);
    without_xml.replace(['/', '\\'], "::")
}