// app.rs
// Setup and run application
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::dpi::PhysicalPosition;
use winit::window::Window;

use crate::render::state::State;

pub struct App {
    event_loop: EventLoop<()>,
    window: Window,
    state: State,
}

impl App {
    pub fn new() -> Self {
        env_logger::init();
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();


        let mut state = pollster::block_on(State::new(&window));
        state.init();

        Self {
            event_loop,
            window,
            state
        }
    }

    pub fn run(mut self) {

        self.event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => if !self.state.input(event) {
                match event {

                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    WindowEvent::CursorMoved { .. } => {
                        let center: [f32; 2] = [
                            self.window.inner_size().width as f32/2.0,
                            self.window.inner_size().height as f32/2.0,
                        ];

                        self.window.set_cursor_position(PhysicalPosition::new(center[0],center[1]))
                            .expect("Failed to reset cursor");
                    }


                    WindowEvent::Resized(physical_size) => {
                        self.state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        self.state.resize(**new_inner_size);
                    }

                    WindowEvent::Focused(true) => {
                        self.window.set_cursor_grab(true)
                            .expect("Couldn't capture cursor!");
                        self.window.set_cursor_visible(false);
                    }

                    WindowEvent::Focused(false) => {
                        self.window.set_cursor_grab(false)
                            .expect("Couldn't release cursor!");
                        self.window.set_cursor_visible(true);
                    }

                    _ => {}
                }
            }

            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                self.state.update();
                match self.state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.state.resize(self.state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                self.window.request_redraw();
            }
            _ => {}
        });
    }
}