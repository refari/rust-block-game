use cgmath::{ElementWise, Matrix4, Point3, Vector3};
use wgpu::{BindGroup, BindGroupLayout, Buffer};
use crate::render::state::State;
use crate::render::traits::Uniform;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub fn look(look_x: f32, look_y: f32) -> Vector3<f32> {
    let mut direction: Vector3<f32> = Vector3::new(0.0,0.0,0.0);

    direction.z = look_y.to_radians().cos() * look_x.to_radians().cos();
    direction.y = look_x.to_radians().sin();
    direction.x = look_y.to_radians().sin() * look_x.to_radians().cos();

    direction
}

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Vector3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: cgmath::Point3<f32>,
        target: cgmath::Vector3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye, target, up, aspect, fovy, znear, zfar,
        }
    }

    // pub fn new_origin() -> Self {
    //     Camera::new(
    //         Point3::new(0.0, 0.0, 0.0),
    //         Vector3::new(0.0, 0.0, 1.0),
    //         Vector3::new(0.0, 1.0, 0.0),
    //         0.5
    //     )
    // }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.eye + self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn move_loc(&mut self, by: Vector3<f32>) {
        use cgmath::InnerSpace;

        let right = self.target.cross(self.up);
        let local_up = self.target.cross(right);


        self.eye += local_up*by.y + right*-by.x + self.target*by.z;
    }

    pub fn look(&mut self, x: f32, y: f32) {
        self.target = look(x,y);
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
}

