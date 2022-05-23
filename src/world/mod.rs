pub mod mesher;

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use cgmath::Vector3;
use wgpu::Buffer;
use crate::render::block::{AIR, Block};
use crate::render::traits::Renderable;
use crate::render::types::Vertex;
use crate::world::mesher::greedy;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_SIZE: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH;

pub struct Chunk {
    pub blocks: [Block; CHUNK_SIZE],
    needs_remesh: bool,
    vert_cache: (Vec<Vertex>, Vec<u32>),
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [AIR; CHUNK_SIZE],
            needs_remesh: false,
            vert_cache: (vec![], vec![]),
        }
    }

    pub fn get_ref(&self, x: usize, y: usize, z: usize) -> &Block {
        if x*CHUNK_WIDTH*CHUNK_WIDTH+y*CHUNK_WIDTH+z > self.blocks.len() {
            println!("Invalid block coords: {}, {}, {}", x, y, z);
        }

        &self.blocks[x*CHUNK_WIDTH*CHUNK_WIDTH+y*CHUNK_WIDTH+z]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        *self.get_ref_mut(x,y,z) = block;
        // self.vert_cache = greedy(&*self);
    }


    fn get_ref_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[x*CHUNK_WIDTH*CHUNK_WIDTH+y*CHUNK_WIDTH+z]
    }




}

impl Renderable for Chunk {
    fn get_vertex_buffer(&self) -> Buffer {
        todo!()
    }

    fn get_index_buffer(&self) -> Buffer {
        todo!()
    }

    fn pre_render(&self) -> Result<(), ()> {
        Ok(())
    }

    fn post_render(&self) -> Result<(), ()> {
        Ok(())
    }

    fn render(&self) -> Result<(), ()> {
        Ok(())
    }
}

struct World {
    chunks: Vec<Chunk>,
    positions: Vec<Vector3<u32>>
}

// impl World {
//     fn new() -> Self {
//         Self {
//             chunks: vec![],
//             positions: vec![]
//         }
//     }
//
//     fn
// }