// texture.rs
// Texture utilities and structs

use image::{DynamicImage, GenericImage, GenericImageView};
use anyhow::*;
use std::default::Default;
use std::{
    path::Path,
    fs,
};
use std::collections::HashMap;
use std::io::Read;
use bytemuck::from_bytes;
use cgmath::num_traits::ToPrimitive;
use crate::render::state::State;

pub struct TextureLoadDescriptor {
    id: String,
    path: Path,
}

#[derive(Default)]
pub struct TextureRegistry {
    textures: HashMap<String, Texture>

}

impl TextureRegistry {
    pub fn add_texture(&mut self, id: &str, texture: Texture) {
        self.textures.insert(id.to_string(), texture);
    }

    pub fn load_and_add(&mut self, state: &State, path: Box<Path>, id: &str) {
        let tex = Texture::from_png(state, path, id)
            .expect("Failed to load texture");
        self.add_texture(id, tex);
    }

    pub fn borrow_texture(&self, id: &str) -> Option<&Texture> {
        self.textures.get(id)
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: Option<wgpu::BindGroup>,
}

// Each corner of a specific texture in the texture atlas.
#[derive(Debug, Copy, Clone)]
pub struct AtlasTexCoords {
    pub bl: [f32; 2],
    pub br: [f32; 2],
    pub tl: [f32; 2],
    pub tr: [f32; 2],
}

pub struct TextureAtlas {
    texture: Texture,
    atlas: DynamicImage,
    textures: Vec<(String, DynamicImage)>,
    width: usize,
    height: usize,
    lookup_table: HashMap<String, AtlasTexCoords>
}

fn best_packing_size(num: usize) -> (u32, u32) {
    let square = num.to_f32().unwrap().sqrt().ceil().to_u32().unwrap();
    (square, square)
}

impl TextureAtlas {
    pub fn new(state: &State, textures: Vec<(String, DynamicImage)>) -> Result<Self> {
        let (w, h) = best_packing_size(textures.len());
        println!("Best size for {} textures is {}, {}", textures.len(), w, h);

        let mut atlas = DynamicImage::new_rgba8(w*16, h*16);
        let mut lookup_table = HashMap::new();

        let sy = 1.0/w as f32;
        let sx = 1.0/h as f32;

        'outer: for x in 0..w {
            for y in 0..h {
                // stop adding textures once we run out of them
                let i = (x*h+y) as usize;
                if i >= textures.len() {
                    break 'outer
                }

                let mut view = atlas.sub_image(x*16, y*16, w*16, h*16);
                view.copy_from(&textures[i].1, 0, 0).expect("Failed to add image!");

                let xf = x as f32 / w as f32;
                let yf = y as f32 / w as f32;

                lookup_table.insert(
                    textures[i].0.clone(),
                    AtlasTexCoords {
                        tl: [xf,    yf   ],
                        tr: [xf+sx, yf   ],
                        bl: [xf,    yf+sy],
                        br: [xf+sx, yf+sy],
                    }
                );

                println!("Adding texture {} at coords ({}, {}) to ({}, {})", textures[i].0, (x as f32)/(w as f32), (y as f32)/(h as f32), (x as f32)/(w as f32)+1.0/(w as f32), (y as f32)/(h as f32)+1.0/(h as f32));
            }
        }

        Ok(Self {
            textures,
            texture: Texture::from_image(state, &atlas, Some("atlas")).expect("Failed to make atlas texture"),
            atlas,
            width: w as usize,
            height: h as usize,
            lookup_table,
        })
    }

    pub fn coords_of(&self, id: &String) -> Result<AtlasTexCoords> {
        if let Some(coords) = self.lookup_table.get(id) {
            return Ok(*coords);
        }

        println!("Tried to find nonexistent texture!");

        Err(anyhow!("This atlas doesn't have this texture"))
    }

    pub fn borrow_atlas_texture(&self) -> &Texture {
        &self.texture
    }
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn image_from_png(path: Box<Path>) -> Result<DynamicImage> {
        let mut file = fs::File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        debug_assert!(!bytes.is_empty(), "Byte buffer was empty");

        Ok(image::load_from_memory(&bytes)?)
    }

    pub fn from_png(
        state: &State,
        path: Box<Path>,
        label: &str
    ) -> Result<Self> {
        let mut file = fs::File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        debug_assert!(bytes.len() > 0, "Byte buffer was empty");

        Self::from_bytes(state, bytes.as_slice(), label)
    }

    fn from_bytes(
        state: &State,
        bytes: &[u8],
        label: &str
    ) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        Self::from_image(state, &image, Some(label))
    }

    fn from_image(
        state: &State,
        img: &DynamicImage,
        label: Option<&str>
    ) -> Result<Self> {
        let rgba = img.as_rgba8().unwrap();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = state.device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );

        state.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = state.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        let bind_group = state.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &state.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        Ok(Self { texture, view, sampler, bind_group: Some(bind_group) })
    }

    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self {
            texture,
            view,
            sampler,
            bind_group: None,
        }
    }
}
