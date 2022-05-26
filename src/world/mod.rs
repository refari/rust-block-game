pub mod mesher;

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::Thread;
use std::time::Instant;
use cgmath::Vector3;
use rayon::prelude::*;
use wgpu::Buffer;
use crate::core::constants::{CHUNK_SIZE, CHUNK_WIDTH};
use crate::render::block::{AIR, Block, BlockRegistry};
use crate::render::texture::TextureAtlas;
use crate::render::traits::Renderable;
use crate::render::types::Vertex;
use crate::world::mesher::greedy;

pub struct Chunk {
    pub blocks: [Block; CHUNK_SIZE],
    pub visible: [bool; CHUNK_SIZE],
    needs_remesh: bool,
    vert_cache: (Vec<Vertex>, Vec<u32>),
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [AIR; CHUNK_SIZE],
            visible: [true; CHUNK_SIZE],
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
        self.update_visible(x,y,z);
        self.needs_remesh = true;

        // self.vert_cache = greedy(&*self);
    }

    pub fn is_visible(&self, x: usize, y: usize, z: usize) -> bool {
        self.visible[x*CHUNK_WIDTH*CHUNK_WIDTH+y*CHUNK_WIDTH+z]
    }

    pub fn set_visible(&mut self, x: usize, y: usize, z: usize, visible: bool) {
        self.visible[x*CHUNK_WIDTH*CHUNK_WIDTH+y*CHUNK_WIDTH+z] = visible;
    }

    pub fn update_visible(&mut self, x: usize, y: usize, z: usize) {
        let mut is_visible = true;
        if x > 0 {
            is_visible = is_visible && self.is_visible(x-1,y,z)
        }
        if x < 15 {
            is_visible = is_visible && self.is_visible(x+1,y,z)
        }
        if y > 0 {
            is_visible = is_visible && self.is_visible(x,y-1,z)
        }
        if y < 15 {
            is_visible = is_visible && self.is_visible(x,y+1,z)
        }
        if z > 0 {
            is_visible = is_visible && self.is_visible(x,y,z-1)
        }
        if z < 15 {
            is_visible = is_visible && self.is_visible(x,y,z+1)
        }

        self.set_visible(x,y,z, is_visible);


        if self.get_ref(x,y,z).transparent && is_visible {
            if x > 0 {
                self.set_visible(x-1,y,z, true);
            }
            if x < 15 {
                self.set_visible(x+1,y,z, true);
            }
            if y > 0 {
                self.set_visible(x,y-1,z, true);
            }
            if y < 15 {
                self.set_visible(x,y+1,z, true);
            }
            if z > 0 {
                self.set_visible(x,y,z-1, true);
            }
            if z < 15 {
                self.set_visible(x,y,z+1, true);
            }
        }
    }

    pub fn get_mesh(&mut self, atlas: &TextureAtlas, palette: &BlockRegistry) -> (Vec<Vertex>, Vec<u32>) {
        if self.needs_remesh {
            self.vert_cache = greedy(self, atlas, palette);
        }
        self.vert_cache.clone()
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

pub struct World {
    chunks: HashMap<Vector3<isize>, Chunk>,
    generator: Option<Arc<dyn WorldGen + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            generator: Some(Arc::new(GenBalls {})),

        }
    }

    pub fn get_chunk(&self, position: Vector3<isize>) -> Result<&Chunk, String> {
        return if let Some(chunk) = self.chunks.get(&position) {
            Ok(chunk)
        } else {
            Err(format!("Chunk {}, {}, {} doesn't exist", position.x, position.y, position.z))
        }
    }

    pub fn get_chunk_or_generate(&mut self, position: Vector3<isize>, palette: &BlockRegistry) -> &Chunk {
        return if self.chunks.get(&position).is_some() {
            self.chunks.get(&position).unwrap()
        } else {
            let chunk = self.generate_chunk(position, palette);

            if let Ok(x) = chunk {
                self.chunks.insert(position, x);
            }

            self.chunks.get(&position).expect("Somehow couldn't get chunk after adding it to world!")
        }
    }

    pub fn generate_chunk(&mut self, position: Vector3<isize>, palette: &BlockRegistry) -> Result<Chunk, String> {
        return if self.chunks.contains_key(&position) {
            Err(String::from("Chunk already exists"))
        } else if self.generator.is_none() {
            Ok(Chunk::new())
        } else {
            let generator = self.generator.as_ref().unwrap();
            let mut chunk = Chunk::new();

            for x in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_WIDTH {
                    for z in 0..CHUNK_WIDTH {
                        chunk.set_block(x,y,z, generator.at((x,y,z), palette));
                    }
                }
            }

            Ok(chunk)
        }
    }

    pub fn make_mesh(&mut self, atlas: &TextureAtlas, palette: &BlockRegistry) -> (Vec<Vertex>, Vec<u32>) {
        let vertices_mutex = Mutex::new(vec![]);
        let indices_mutex = Mutex::new(vec![]);


        let chunks_completed = AtomicUsize::from(0);
        self.chunks.par_iter_mut().for_each(|(p, i)|{
            let overall_1 = Instant::now();
            let get_mesh1 = Instant::now();
            let (mut _vertices, mut _indices) = i.get_mesh(atlas, palette);
            let get_mesh2 = Instant::now();
            println!("It took {:?} to construct a mesh for this chunk", get_mesh2-get_mesh1);
            {
                let i1_1 = Instant::now();
                let mut verts_wait = _vertices.into_par_iter().map(|mut x| {
                    x.position[0] += (p.x*CHUNK_WIDTH as isize) as f32;
                    x.position[1] += (p.y*CHUNK_WIDTH as isize) as f32;
                    x.position[2] += (p.z*CHUNK_WIDTH as isize) as f32;
                    x
                }).collect();
                let i1_2 = Instant::now();
                println!("It took {:?} to process vertices", i1_2-i1_1);

                let i2_1 = Instant::now();
                vertices_mutex.lock().unwrap().append(
                    &mut verts_wait
                );
                let i2_2 = Instant::now();
                println!("And {:?} to append them", i2_2-i2_1);
            }

            {
                let mut indices = indices_mutex.lock().unwrap();
                if indices.is_empty() {
                    indices.append(&mut _indices);
                } else {
                    let max = indices.iter().max().unwrap();

                    _indices = _indices.into_par_iter().map(|x| {
                        x + max + 1
                    }).collect();

                    indices.append(&mut _indices);
                }
            }
            chunks_completed.fetch_add(1, Ordering::SeqCst);
            // println!("Completed chunk {}", chunks_completed.load(Ordering::SeqCst));
            let overall_2 = Instant::now();
            println!("It took {:?} to make meshes for chunk #{}", overall_2-overall_1, chunks_completed.load(Ordering::SeqCst));
        });

        let vertices = vertices_mutex.lock().unwrap().clone();
        let indices = indices_mutex.lock().unwrap().clone();



        (vertices, indices)
    }
}

trait WorldGen {
    fn at(&self, coords: (usize, usize, usize), palette: &BlockRegistry) -> Block;
}

struct GenBalls;

impl GenBalls {
    fn in_sphere(x: usize, y: usize, z: usize, r: f32) -> bool {
        let radius = ((
            (x as i32 - (CHUNK_WIDTH/2) as i32).pow(2) +
            (y as i32 - (CHUNK_WIDTH/2) as i32).pow(2) +
            (z as i32 - (CHUNK_WIDTH/2) as i32).pow(2)) as f32
        ).sqrt();
        radius < r
    }
}

impl WorldGen for GenBalls {
    fn at(&self, coords: (usize, usize, usize), palette: &BlockRegistry) -> Block {
        let (x,y,z) = coords;


        if GenBalls::in_sphere(x,y,z, 18.0) {
            if GenBalls::in_sphere(x,y+1,z, 18.0) {
                palette.block("dirt")
            } else {
                palette.block("grass")
            }
        } else {
            palette.block("air")
        }
    }
}

struct GenFullRandom;

impl WorldGen for GenFullRandom {
    fn at(&self, coords: (usize, usize, usize), palette: &BlockRegistry) -> Block {
        palette.block("dirt")
    }
}