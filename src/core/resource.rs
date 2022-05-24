// resource.rs
// Resource manager and utils.

use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use anyhow::Result;
use image::DynamicImage;

#[derive(Eq, PartialEq)]
pub enum ResType {
    Image, Sound, Shader,
}

pub struct ResourceManager {
    resources: HashMap<String, (ResType, Box<dyn Resource>)>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        ResourceManager::new()
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new()
        }
    }

    pub fn find_images(&self, filters: Vec<String>) -> Vec<&ImageResource> {
        let images = self.resources
            .iter()
            .filter(|(x,(y,z))| {
                if *y != ResType::Image {
                    return false;
                }
                let mut contains_all = true;
                filters.iter().for_each(|f| {
                    contains_all = contains_all && x.contains(f);
                });
                contains_all
            }).map(|(x, _)| {
                self.get_image(x.as_str()).unwrap()
            }).collect();

        images
    }

    pub fn add_resource(&mut self, id: String, res_type: ResType, res: Box<dyn Resource>) {
        self.resources.insert(id, (res_type, res));
    }

    pub fn get_image(&self, id: &str) -> Option<&ImageResource> {
        let res = self.resources.get(id);
        if res.is_some() {
            let (t, _res) = res.unwrap();
            if *t != ResType::Image {
                return None;
            }
            let res = match _res.as_any().downcast_ref::<ImageResource>() {
                Some(b) => Some(b),
                None => panic!("Resource marked as image isn't an image!")
            };
            return res;
        }
        None
    }

    pub fn get_shader(&self, id: &str) -> Option<&ShaderResource> {
        let res = self.resources.get(id);
        if res.is_some() {
            let (t, _res) = res.unwrap();
            if *t != ResType::Shader {
                return None;
            }
            let res = match _res.as_any().downcast_ref::<ShaderResource>() {
                Some(b) => Some(b),
                None => panic!("Resource marked as shader isn't an shader!")
            };
            return res;
        }
        None
    }
}

pub trait Resource {
    fn get_generic_metadata(&self) -> GenericMetadata;
    fn id(&self) -> &String;
    fn reload(&mut self) -> Result<ReloadInfo>;
    fn is_loaded(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
}

// Image resource

pub struct ImageResource {
    path: Box<Path>,
    generic_metadata: GenericMetadata,
    id: String,

    image: Option<DynamicImage>,
}

impl ImageResource {
    pub fn new(id: String, path: Box<Path>) -> Self {
        let mut res = Self {
            path,
            generic_metadata: GenericMetadata {},
            id,
            image: None,
        };
        res.reload().expect("Couldn't load image");
        res
    }

    pub fn get(&self) -> &DynamicImage {
        self.image.as_ref()
            .expect("Image loaded successfully, but it couldn't be unwrapped!")
    }
}

impl Resource for ImageResource {
    fn get_generic_metadata(&self) -> GenericMetadata {
        self.generic_metadata
    }

    fn id(&self) -> &String {
        &self.id
    }

    fn reload(&mut self) -> Result<ReloadInfo> {
        let mut file = fs::File::open(&self.path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        debug_assert!(!bytes.is_empty(), "Byte buffer was empty");

        self.image = Some(image::load_from_memory(&bytes)?);
        Ok(ReloadInfo {})
    }

    fn is_loaded(&self) -> bool {
        self.image.is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Sound resource

struct SoundResource {
    // TODO
}

// Shader resource

pub struct ShaderResource {
    path: Box<Path>,
    generic_metadata: GenericMetadata,
    id: String,

    shader: Option<String>
}

impl Resource for ShaderResource {

    fn get_generic_metadata(&self) -> GenericMetadata {
        self.generic_metadata
    }

    fn id(&self) -> &String {
        &self.id
    }

    fn reload(&mut self) -> Result<ReloadInfo> {
        let mut file = fs::File::open(&self.path)?;
        let mut str = String::new();
        file.read_to_string(&mut str).expect("Couldn't read file.");
        debug_assert!(!str.is_empty(), "File is empty.");

        self.shader = Some(str);

        Ok(ReloadInfo {})
    }

    fn is_loaded(&self) -> bool {
        self.shader.is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ShaderResource {
    pub fn new(id: String, path: Box<Path>) -> Self {
        let mut res = Self {
            path,
            generic_metadata: GenericMetadata {},
            id,
            shader: None,
        };
        res.reload().expect("Couldn't load shader");
        res
    }

    pub fn get(&self) -> &String {
        self.shader.as_ref()
            .expect("Shader was reloaded but failed to unwrap!")
    }

    pub fn make_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some(self.id.as_str()),
                source: wgpu::ShaderSource::Wgsl(
                    self.shader.as_ref().unwrap().as_str().into()
                )
            }
        )
    }
}

#[derive(Copy, Clone)]
pub struct ReloadInfo {

}

#[derive(Copy, Clone)]
pub struct GenericMetadata {

}