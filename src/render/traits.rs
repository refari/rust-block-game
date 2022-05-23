use wgpu::Buffer;

/// Uniforms in WGPU must make a bind group layout, bind group, and a
pub trait Uniform {
    fn get_bind_group(&self) -> wgpu::BindGroup;
    fn get_buffer(&self) -> wgpu::Buffer;
}

pub trait Renderable {
    fn get_vertex_buffer(&self) -> Buffer;
    fn get_index_buffer(&self) -> Buffer;
    fn pre_render(&self) -> Result<(), ()>;
    fn post_render(&self) -> Result<(), ()>;
    fn render(&self) -> Result<(), ()>;
}
