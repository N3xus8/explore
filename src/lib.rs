pub mod app;
pub use app::App;
pub mod state;
pub mod utils;
pub mod web_utils;
pub mod pipeline;
pub mod vertex;
pub mod texture;
pub mod camera;
pub mod model;
pub mod resources;
pub mod extra;
pub mod depth_stencil;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
 
