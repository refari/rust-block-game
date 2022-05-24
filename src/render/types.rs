// types.rs
// Useful generic types that will be used often.
extern crate bytemuck;

use anyhow::*;
use block_mesh::OrientedBlockFace;
use cgmath::{ElementWise, Vector2, Vector3};
use crate::player::camera::Camera;


#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [f32; 2],
    pub normals: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn from_pos(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x,y,z],
            texcoord: [0.0,0.0],
            normals: [0.0,0.0,0.0],
        }
    }
}