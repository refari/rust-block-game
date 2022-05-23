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

pub struct Face {
    pub vertices: [Vertex; 4],
    pub indices: [u32; 6],
    pub useless: bool,
}

impl Face {
    // pub fn plane(pos: Vector3<f32>, dir: Vector3<f32>, size: Vector2<f32>) -> Result<Self> {
    //     if size.x * size.y == 0.0 {
    //         return Ok(Face::useless());
    //     }
    //
    //     let a = dir.cross(Vector3::new(0.0, 1.0, 0.0)).mul_element_wise(size);
    //     let b = dir.cross(a).mul_element_wise(size);
    //
    //     let vertices = [
    //         Vertex { position: (a+b+pos).into(),  texcoord: [0.0, 0.0]},
    //         Vertex { position: (pos).into(), texcoord: [1.0, 0.0]},
    //         Vertex { position: (a-b+pos).into(),  texcoord: [0.0, 1.0]},
    //         Vertex { position: (pos).into(), texcoord: [1.0, 1.0]},
    //     ];
    //
    //     Ok(Self {
    //         vertices,
    //         indices: [0,1,2, 3,2,1],
    //         useless: false,
    //     })
    // }

    pub fn useless() -> Self {
        Self {
            vertices: [
                Vertex { position: [0.0,0.0,0.0], texcoord: [0.0,0.0], normals: [0.0,0.0,0.0]},
                Vertex { position: [0.0,0.0,0.0], texcoord: [0.0,0.0], normals: [0.0,0.0,0.0]},
                Vertex { position: [0.0,0.0,0.0], texcoord: [0.0,0.0], normals: [0.0,0.0,0.0]},
                Vertex { position: [0.0,0.0,0.0], texcoord: [0.0,0.0], normals: [0.0,0.0,0.0]},
            ],
            indices: [0,0,0, 0,0,0],
            useless: true,
        }
    }
}