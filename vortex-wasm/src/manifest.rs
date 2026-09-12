use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
pub struct VortexManifest {
    pub entries: Vec<VortexAssetEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VortexAssetEntry {
    pub format: String,
    pub path: String,
    pub size: u64,
    pub sha256: String,
    pub classification: String,
    #[serde(default)]
    pub nodes: Vec<String>,
    #[serde(default)]
    pub meshes: Vec<String>,
    #[serde(default)]
    pub animations: Vec<String>,
}

#[derive(Default, TypePath)]
pub struct VortexManifestLoader;

#[derive(Debug, Error)]
pub enum VortexManifestLoaderError {
    #[error("could not read Vortex manifest: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse Vortex manifest: {0}")]
    Json(#[from] serde_json::Error),
}

impl AssetLoader for VortexManifestLoader {
    type Asset = VortexManifest;
    type Settings = ();
    type Error = VortexManifestLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
