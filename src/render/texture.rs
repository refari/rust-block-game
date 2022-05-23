use image::GenericImageView;
use anyhow::*;
use wgpu::{BindGroupLayout, Device, SurfaceConfiguration, TextureUsages};
use std::default::Default;
use std::{
    path::Path,
    fs,
};
use std::collections::HashMap;
use std::io::Read;
use bytemuck::from_bytes;
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
struct AtlasTexCoords {
    bl: [f32; 2],
    br: [f32; 2],
    tl: [f32; 2],
    tr: [f32; 2],
}

pub struct TextureAtlas {
    atlas: Texture,
    textures: Vec<Texture>,
}

impl TextureAtlas {
    pub fn new(textures: Vec<Texture>) -> Self {
        Self
    }
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
        img: &image::DynamicImage,
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

    pub fn create_depth_texture(device: &Device, config: &SurfaceConfiguration, label: &str) -> Self {
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
