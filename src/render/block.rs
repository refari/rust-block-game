use std::collections::HashMap;
use std::hash::Hash;
use block_mesh::{MergeVoxel, Voxel};
use cgmath::Vector3;
use crate::render::texture::Texture;
use crate::render::types::{Face, Vertex};

const BLOCK_FACES_DIRS: [Vector3<f32>; 6] = [
    Vector3::new(1.0, 0.0, 0.0),
    Vector3::new(-1.0, 0.0, 0.0),
    Vector3::new(0.0, 1.0, 0.0),
    Vector3::new(0.0, -1.0, 0.0),
    Vector3::new(0.0, 0.0, 1.0),
    Vector3::new(0.0, 0.0, -1.0),
];

#[derive(Default)]
pub struct BlockRegistry {
    keys: HashMap<String, u32>,
    blocks: HashMap<u32, BlockDescriptor>,
}

impl BlockRegistry {

    pub fn add_block(&mut self, block: BlockDescriptor) {
        let max = self.keys.values().max();
        let next = if max.is_none() {
            0
        } else {
            max.unwrap() + 1
        };
        self.keys.insert(block.id.clone(), next);
        self.blocks.insert(next, block);
    }

    pub fn get_str(&self, id: &str) -> &BlockDescriptor {
        let key = self.keys.get(id).expect("Tried to get a nonexistent block");
        self.blocks.get(key).unwrap()
    }

    pub fn get_uint(&self, id: &u32) -> &BlockDescriptor {
        self.blocks.get(id).expect("Tried to get a nonexistent block")
    }

    pub fn block(&self, id: &str) -> Block {
        let desc = self.get_str(id);
        Block {
            desc_index: *self.keys.get(id).unwrap(),
            invisible: desc.invisible,
            transparent: desc.transparent,
        }
    }
}

pub struct BlockDescriptor {
    pub id: String,
    pub invisible: bool,
    pub transparent: bool,
    pub top_texture: Option<String>,
    pub bottom_texture: Option<String>,
    pub side_textures: [Option<String>; 4],
}

impl BlockDescriptor {
    pub fn new(
        id: &str,
        invisible: bool,
        transparent: bool,
        top_texture: Option<&str>,
        bottom_texture: Option<&str>,
        _side_textures: [Option<&str>; 4]
    ) -> Self {

        Self {
            id: id.to_string(),
            invisible,
            transparent,
            top_texture: top_texture.map(|tex| { tex.to_string() }),
            bottom_texture: bottom_texture.map(|tex| { tex.to_string() }),
            side_textures: [
                _side_textures[0].map(|tex| {tex.to_string()}),
                _side_textures[1].map(|tex| {tex.to_string()}),
                _side_textures[2].map(|tex| {tex.to_string()}),
                _side_textures[3].map(|tex| {tex.to_string()}),
            ]
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Block {
    pub desc_index: u32,
    pub invisible: bool,
    pub transparent: bool,
}

pub const AIR: Block = Block {
    desc_index: 0,
    invisible: true,
    transparent: true,
};

impl Voxel for Block {
    fn is_empty(&self) -> bool {
        self.invisible
    }

    fn is_opaque(&self) -> bool {
        !self.transparent
    }
}

impl MergeVoxel for Block {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}