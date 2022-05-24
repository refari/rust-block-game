// mesher.rs
// Naive mesh generator outputting texture coordinates and vertex normals.
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::collections::btree_map::BTreeMap;
use std::iter::Map;
use image::imageops::index_colors;
use crate::render::block::BlockRegistry;
use crate::render::texture::{AtlasTexCoords, TextureAtlas};
use crate::render::types::Vertex;
use crate::world::*;

enum Dir {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

pub fn greedy(chunk: &Chunk, atlas: &TextureAtlas, block_registry: &BlockRegistry) -> (Vec<Vertex>, Vec<u32>) {
    const CHUNK_WIDTH_B: usize = CHUNK_WIDTH+1;
    fn lin(x: usize, y: usize, z: usize) -> u32 {
        (x*CHUNK_WIDTH_B*CHUNK_WIDTH_B + y*CHUNK_WIDTH_B + z) as u32
    }

    fn add_face(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, pos: (f32, f32, f32), dir: Dir, texcoords: AtlasTexCoords) {
        let (x,y,z) = pos;
        let face_start = vertices.len() as u32;

        match dir {
            Dir::Up => {
                vertices.push(Vertex::from_pos(x,     y+1.0, z));
                vertices.push(Vertex::from_pos(x+1.0, y+1.0, z));
                vertices.push(Vertex::from_pos(x,     y+1.0, z+1.0));
                vertices.push(Vertex::from_pos(x+1.0, y+1.0, z+1.0));

                let len = vertices.len();
                let slice_face = &mut vertices[len-4..len];
                assert_eq!(slice_face.len(), 4);

                slice_face[0].texcoord = texcoords.tr;
                slice_face[1].texcoord = texcoords.tl;
                slice_face[2].texcoord = texcoords.br;
                slice_face[3].texcoord = texcoords.bl;

                for i in slice_face {
                    i.normals = [0.0, 1.0, 0.0];
                }
            }
            Dir::Down => {
                vertices.push(Vertex::from_pos(x,     y, z));
                vertices.push(Vertex::from_pos(x,     y, z+1.0));
                vertices.push(Vertex::from_pos(x+1.0, y, z));
                vertices.push(Vertex::from_pos(x+1.0, y, z+1.0));

                let len = vertices.len();
                let slice_face = &mut vertices[len-4..len];
                assert_eq!(slice_face.len(), 4);

                slice_face[0].texcoord = texcoords.tr;
                slice_face[1].texcoord = texcoords.tl;
                slice_face[2].texcoord = texcoords.br;
                slice_face[3].texcoord = texcoords.bl;

                for i in slice_face {
                    i.normals = [0.0, -1.0, 0.0];
                }
            }
            Dir::Right => {
                vertices.push(Vertex::from_pos(x, y,     z));
                vertices.push(Vertex::from_pos(x, y+1.0, z));
                vertices.push(Vertex::from_pos(x, y,     z+1.0));
                vertices.push(Vertex::from_pos(x, y+1.0, z+1.0));

                let len = vertices.len();
                let slice_face = &mut vertices[len-4..len];
                assert_eq!(slice_face.len(), 4);

                slice_face[0].texcoord = texcoords.br;
                slice_face[1].texcoord = texcoords.tr;
                slice_face[2].texcoord = texcoords.bl;
                slice_face[3].texcoord = texcoords.tl;

                for i in slice_face {
                    i.normals = [-1.0, 0.0, 0.0];
                }
            }
            Dir::Left => {
                vertices.push(Vertex::from_pos(x+1.0, y,     z));
                vertices.push(Vertex::from_pos(x+1.0, y,     z+1.0));
                vertices.push(Vertex::from_pos(x+1.0, y+1.0, z));
                vertices.push(Vertex::from_pos(x+1.0, y+1.0, z+1.0));

                let len = vertices.len();
                let slice_face = &mut vertices[len-4..len];
                assert_eq!(slice_face.len(), 4);

                slice_face[0].texcoord = texcoords.bl;
                slice_face[1].texcoord = texcoords.br;
                slice_face[2].texcoord = texcoords.tl;
                slice_face[3].texcoord = texcoords.tr;

                for i in slice_face {
                    i.normals = [1.0, 0.0, 0.0];
                }
            }
            Dir::Back => {
                vertices.push(Vertex::from_pos(x,     y,     z+1.0));
                vertices.push(Vertex::from_pos(x,     y+1.0, z+1.0));
                vertices.push(Vertex::from_pos(x+1.0, y,     z+1.0));
                vertices.push(Vertex::from_pos(x+1.0, y+1.0, z+1.0));

                let len = vertices.len();
                let slice_face = &mut vertices[len-4..len];
                assert_eq!(slice_face.len(), 4);

                slice_face[0].texcoord = texcoords.br;
                slice_face[1].texcoord = texcoords.tr;
                slice_face[2].texcoord = texcoords.bl;
                slice_face[3].texcoord = texcoords.tl;

                for i in slice_face {
                    i.normals = [0.0, 0.0, 1.0];
                }
            }
            Dir::Front => {
                vertices.push(Vertex::from_pos(x,     y,     z));
                vertices.push(Vertex::from_pos(x+1.0, y,     z));
                vertices.push(Vertex::from_pos(x,     y+1.0, z));
                vertices.push(Vertex::from_pos(x+1.0, y+1.0, z));

                let len = vertices.len();
                let slice_face = &mut vertices[len-4..len];
                assert_eq!(slice_face.len(), 4);

                slice_face[0].texcoord = texcoords.br;
                slice_face[1].texcoord = texcoords.bl;
                slice_face[2].texcoord = texcoords.tr;
                slice_face[3].texcoord = texcoords.tl;

                for i in slice_face {
                    i.normals = [0.0, 0.0, -1.0];
                }
            }
        }

        indices.push(face_start+1);
        indices.push(face_start);
        indices.push(face_start+2);
        indices.push(face_start+1);
        indices.push(face_start+2);
        indices.push(face_start+3);
    }

    // println!("{} points!", counter);

    // let mut current_index = 0u32;
    let mut new_indices: Vec<u32> = Vec::new();
    let mut new_vertices: Vec<Vertex> = Vec::new();

    for x in 0..CHUNK_WIDTH {
        for y in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let block = chunk.get_ref(x,y,z);
                let desc = block_registry.get_uint(&block.desc_index);

                if block.invisible {
                    continue;
                }

                // positives

                if y + 1 < CHUNK_WIDTH && chunk.get_ref(x, y + 1, z).transparent {
                    let coords = atlas.coords_of(desc.top_texture.as_ref().unwrap());
                    if coords.is_ok() {
                        add_face(&mut new_vertices, &mut new_indices, (x as f32, y as f32, z as f32), Dir::Up, coords.unwrap());
                    }
                }

                if x + 1 < CHUNK_WIDTH && chunk.get_ref(x + 1, y, z).transparent {
                    let coords = atlas.coords_of(desc.side_textures[3].as_ref().unwrap());
                    if coords.is_ok() {
                        add_face(&mut new_vertices, &mut new_indices, (x as f32, y as f32, z as f32), Dir::Left, coords.unwrap());
                    }
                }

                if z + 1 < CHUNK_WIDTH && chunk.get_ref(x, y, z + 1).transparent {
                    let coords = atlas.coords_of(desc.side_textures[2].as_ref().unwrap());
                    if coords.is_ok() {
                        add_face(&mut new_vertices, &mut new_indices, (x as f32, y as f32, z as f32), Dir::Back, coords.unwrap());
                    }
                }

                // negatives

                if y - 1 > 0 && chunk.get_ref(x, y - 1, z).transparent {
                    let coords = atlas.coords_of(desc.bottom_texture.as_ref().unwrap());
                    if coords.is_ok() {
                        add_face(&mut new_vertices, &mut new_indices, (x as f32, y as f32, z as f32), Dir::Down, coords.unwrap());
                    }
                }

                if x - 1 > 0 && chunk.get_ref(x - 1, y, z).transparent {
                    let coords = atlas.coords_of(desc.side_textures[1].as_ref().unwrap());
                    if coords.is_ok() {
                        add_face(&mut new_vertices, &mut new_indices, (x as f32, y as f32, z as f32), Dir::Right, coords.unwrap());
                    }
                }

                if z - 1 > 0 && chunk.get_ref(x, y, z - 1).transparent {
                    let coords = atlas.coords_of(desc.side_textures[0].as_ref().unwrap());
                    if coords.is_ok() {
                        add_face(&mut new_vertices, &mut new_indices, (x as f32, y as f32, z as f32), Dir::Front, coords.unwrap());
                    }
                }
            }
        }
    }

    (new_vertices, new_indices)
}