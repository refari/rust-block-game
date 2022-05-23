use std::rc::Rc;
use cgmath::{Point3, Vector2, Vector3};
use wgpu::{Buffer, Queue};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use crate::player::camera::{Camera, CameraUniform, look};
use crate::render::state::State;

pub mod camera;

pub struct PlayerManager {
    players: Vec<Player>,
}

pub struct Player {
    controller: PlayerController,
    camera: Camera,
    uniform: CameraUniform,
    window_size: PhysicalSize<u32>,
    prev_pos: Option<PhysicalPosition<f64>>,
}

impl Player {
    pub fn new(window_size: PhysicalSize<u32>, state: &State) -> Self {
        let mut controller = PlayerController::new(state);
        let mut camera = controller.make_camera();

        let uniform = CameraUniform {
            view_proj: camera.build_view_projection_matrix().into(),
        };
        Self {
            controller,
            camera,
            uniform,
            prev_pos: None,
            window_size,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // pick out events relevant to the player
        match event {
            // keyboard events are important
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.controller.keys.forward = is_pressed;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.controller.keys.backward = is_pressed;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.controller.keys.right = is_pressed;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.controller.keys.left = is_pressed;
                        true
                    }

                    _ => { false }
                }
            }

            WindowEvent::CursorMoved {

                position,

                ..
            } => {
                if self.prev_pos.is_none() {
                    self.prev_pos = Some(position.clone());
                }

                let delta_x = position.x - (self.window_size.width as f64)/2.0;
                let delta_y = position.y - (self.window_size.height as f64)/2.0;

                self.controller.lookx += delta_y as f32 * 0.01;
                self.controller.looky += delta_x as f32 * 0.01;


                if self.controller.lookx > 90.0 {
                    self.controller.lookx = 90.0;
                }

                if self.controller.lookx < -90.0 {
                    self.controller.lookx = -90.0;
                }

                self.camera.look(self.controller.lookx, self.controller.looky);

                self.prev_pos = Some(position.clone());

                false
            }

            _ => { false }
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.window_size = size;
    }

    pub fn update(&mut self, queue: &Queue, buffer: &Buffer) {
        self.controller.update(&mut self.camera);

        self.uniform = CameraUniform {
            view_proj: self.camera.build_view_projection_matrix().into(),
        };
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

#[derive(Default)]
struct Keys {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

struct PlayerController {
    pub position: Point3<f32>,
    pub lookx: f32,
    pub looky: f32,
    aspect: f32,
    keys: Keys,
}

impl PlayerController {
    pub fn new(state: &State) -> Self {
        Self {
            position: Point3::new(0.0,0.0,0.0),
            lookx: 0.0,
            looky: 0.0,
            aspect: state.size.width as f32 / state.size.height as f32,
            keys: Keys::default(),
        }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        if self.keys.forward {
            camera.move_loc(Vector3::new(0.0, 0.0, 0.1));
        } else if self.keys.backward {
            camera.move_loc(Vector3::new(0.0, 0.0, -0.1));
        }

        if self.keys.right {
            camera.move_loc(Vector3::new(0.1, 0.0, 0.0));
        } else if self.keys.left {
            camera.move_loc(Vector3::new(-0.1, 0.0, 0.0));
        }
    }

    pub fn make_camera(&self) -> Camera {
        Camera::new(
            self.position,
            look(self.lookx, self.looky),
            Vector3::new(0.0, 1.0, 0.0),
            self.aspect,
            45.0,
            0.1,
            100.0
        )
    }
}