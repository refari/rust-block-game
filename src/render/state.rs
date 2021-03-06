// state.rs
// Application state
extern crate winit;

use std::mem::size_of;
use std::path::Path;
use std::time::{Duration, Instant};
use cgmath::Vector3;
use image::DynamicImage;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::{
    event::*,
    window::Window,
};
use winit::dpi::PhysicalSize;

use crate::render::texture::{Texture, TextureAtlas, TextureRegistry};
use crate::player::camera::{Camera, CameraUniform};
use crate::player::{Player, PlayerManager};

use crate::render::{
    types::{Vertex},
};
use crate::render::block::{Block, BlockDescriptor, BlockRegistry};

use rayon::prelude::*;
use crate::core::resource::{ImageResource, Resource, ResourceManager, ResType, ShaderResource};

use crate::world::{Chunk, World};

pub struct State {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_vertices: Option<u32>,
    player: Option<Player>,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    window_size: PhysicalSize<u32>,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    // textures: TextureRegistry,
    resources: ResourceManager,
    textures: Option<TextureAtlas>,
    blocks: BlockRegistry,
    world: Option<World>,
}

impl State {
    // Creating some of the WGPU types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::GL | wgpu::Backends::METAL);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let mut resources = ResourceManager::new();

        resources.add_resource(
            String::from("Main Shader"),
            ResType::Shader,
            Box::new(ShaderResource::new(
                String::from("Main Shader"),
                Path::new("res/shaders/shader.wgsl").into()
            ))
        );

        let shader = resources.get_shader("Main Shader")
            .expect("Couldn't get shader")
            .make_module(&device);

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            // SamplerBindingType::Comparison is only for TextureSampleType::Depth
                            // SamplerBindingType::Filtering if the sample_type of the texture is:
                            //     TextureSampleType::Float { filterable: true }
                            // Otherwise you'll get an error.
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            }
        );

        let _camera = Camera::new(
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            (0.0, 0.0, 0.0).into(),
            // which way is "up"
            Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let camera_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Camera Buffer"),
                size: size_of::<CameraUniform>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc(),
                    ],
                },
                fragment: Some(wgpu::FragmentState { // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState { // 4.
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }), // 1.
                multisample: wgpu::MultisampleState {
                    count: 1, // 2.
                    mask: !0, // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            },
        );


        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            depth_texture,
            vertex_buffer: None,
            index_buffer: None,
            num_vertices: None,
            player: None,
            window_size: size,
            camera_bind_group,
            camera_buffer,
            texture_bind_group_layout,
            camera_bind_group_layout,
            resources,
            textures: None,
            blocks: BlockRegistry::default(),
            world: None,
        }
    }

    pub fn init(&mut self) {
        self.player = Some(Player::new(self.window_size, self));

        self.resources.add_resource(
            String::from("^terrain_grass_top"),
            ResType::Image,
            Box::new(
                ImageResource::new(
                    String::from("grass_top"),
                    Path::new("res/images/grass/grass_top.png").into()
                )
            )
        );

        self.resources.add_resource(
            String::from("^terrain_grass_bottom"),
            ResType::Image,
            Box::new(
                ImageResource::new(
                    String::from("grass_bottom"),
                    Path::new("res/images/grass/grass_bottom.png").into()
                )
            )
        );

        self.resources.add_resource(
            String::from("^terrain_grass_side"),
            ResType::Image,
            Box::new(
                ImageResource::new(
                    String::from("grass_side"),
                    Path::new("res/images/grass/grass_side.png").into()
                )
            )
        );

        let matches = self.resources.find_images(vec![String::from("^terrain")]);
        let textures: Vec<(String, DynamicImage)> = matches.iter().map(|x| {
            let y = x.id();
            let z = x.get();
            (y.clone(), z.clone())
        }).collect();

        let atlas = TextureAtlas::new(
            self,
            textures,
        ).expect("Couldn't create atlas");
        self.textures = Some(atlas);

        let air = BlockDescriptor::new(
            "air",
            true,
            true,
            None,
            None,
            [None; 4],
        );

        let grass = BlockDescriptor::new(
            "grass",
            false,
            false,
            Some("grass_top"),
            Some("grass_bottom"),
            [Some("grass_side"); 4],
        );

        let dirt = BlockDescriptor::new(
            "dirt",
            false,
            false,
            Some("grass_bottom"),
            Some("grass_bottom"),
            [Some("grass_bottom"); 4],
        );


        self.blocks.add_block(air);
        self.blocks.add_block(grass);
        self.blocks.add_block(dirt);

        self.world = Some(World::new());


        let mut _world = self.world.as_mut().unwrap();

        let before_generation = Instant::now();
        for x in -2..2 {
            for y in -2..2 {
                for z in -2..2 {
                    _world.get_chunk_or_generate(Vector3::new(x,y,z), &self.blocks);
                }
            }
        }
        let after_generation = Instant::now();
        println!("Took {:?} to generate blocks", after_generation-before_generation);


        let before_mesh_multi = Instant::now();
        let (vertices, indices) = _world.make_mesh(
            self.textures.as_ref().unwrap(),
            &self.blocks
        );
        let after_mesh_multi = Instant::now();
        println!("Took {:?} seconds to generate mesh (multi-threaded)", after_mesh_multi-before_mesh_multi);


        // let before_mesh_single = Instant::now();
        // let (vertices, indices) = _world.make_mesh_single(
        //     self.textures.as_ref().unwrap(),
        //     &self.blocks
        // );
        // let after_mesh_single = Instant::now();
        // println!("Took {:?} seconds to generate mesh (single-threaded)", after_mesh_single-before_mesh_single);

        let num_indices = indices.len();
        let num_vertices = vertices.len();



        self.num_vertices = Some(num_indices as u32);

        self.vertex_buffer = Some(self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        self.index_buffer = Some(self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        }));
        
        println!("{} vertices", num_vertices);
        println!("{} indices", num_indices);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.player.as_mut().unwrap().resize(new_size);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.player.as_mut().unwrap().input(event)
    }

    pub fn update(&mut self) {
        self.player.as_mut().unwrap().update(&self.queue, &self.camera_buffer);

    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(
                0,
                self.textures.as_ref().unwrap().borrow_atlas_texture().bind_group.as_ref().unwrap(),
                &[]
            );
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(self.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_vertices.unwrap(), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}



