use crate::model::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ModelRef(usize);
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct RigRef(usize);
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct AnimRef(usize);

pub struct Object2d {
    pub bg: usize,
    pub visible: bool,
    pub verts: [VertexTwoD],
}
pub struct Asset2d(pub PathBuf);
pub struct Assets {
    asset_root: PathBuf,
    models: HashMap<ModelRef, Model>,
}
impl Assets {
    pub fn new(asset_root: impl AsRef<Path>) -> Self {
        // ... register filesystem watchers with crate notify = "4.0.15":
        Self {
            asset_root: asset_root.as_ref().to_owned(),
            models: HashMap::new(),
        }
    }
    pub fn load_model(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        model: impl AsRef<Path>,
    ) -> ModelRef {
        let mref = ModelRef(self.models.len());
        let ar = &self.asset_root;
        self.models.insert(
            mref,
            Model::load(device, queue, layout, ar.join(&model)).unwrap(),
        );
        mref
    }
    pub fn get_model(&self, model: ModelRef) -> Option<&Model> {
        self.models.get(&model)
    }
}
