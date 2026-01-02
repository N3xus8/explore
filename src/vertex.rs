
use anyhow::*;
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, prelude::*};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
   pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {     // Position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {     // Color
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}
 
 
// pub const VERTICES: &[Vertex] = &[
//     Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
//     Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
//     Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
// ];
// pub const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
// ];

// pub const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.99240386], }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.56958647], }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.05060294], }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.1526709], }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.7347359], }, // E
// ];

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.868241, 4.9240386, 0.0],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    Vertex {
        position: [-4.9513406, 0.6958647, 0.0],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    Vertex {
        position: [-2.1918549, -4.4939706, 0.0],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    Vertex {
        position: [3.5966997, -3.473291, 0.0],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    Vertex {
        position: [4.414737, 2.347359, 0.0],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
 
pub fn create_vertex_buffer(
        device: &wgpu::Device,      
    ) -> Result<wgpu::Buffer> {

    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    Ok(vertex_buffer)
}

pub fn create_index_buffer(
         device: &wgpu::Device,        
        ) -> Result<wgpu::Buffer> {

    let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
        }
    );

    Ok(index_buffer)
}


const NUM_INSTANCES_PER_ROW: u32 = 2;
//const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);
const SPACE_BETWEEN: f32 = 3.0;

pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

impl Instance {
        
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
    pub fn to_raw_with_scale(&self, scale: f32) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation) * Matrix4::from_nonuniform_scale(scale, scale, 1.0)).into(),
        }
    }
    pub fn generate_instances() -> Vec<Instance> {
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();
        
        instances

    }

    pub fn generate_instance(
        x: f32,
        y: f32,
        z: f32,
        angle: f32,
    ) -> Instance {
             let position = cgmath::Vector3 { x, y, z };


             let rotation=   cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(angle));
             

                Instance {
                    position, rotation,
                }
    }

    pub fn translation(&self) -> cgmath::Matrix4<f32> {
        Matrix4::from_translation(self.position)
    }

    pub fn rotation(&self) -> cgmath::Matrix4<f32> {
        Matrix4::from(self.rotation)
    }

    pub fn transform(&self)  -> cgmath::Matrix4<f32> {

        self.translation() * self.rotation() 
    
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub fn create_instance_buffer(device: &wgpu::Device, instance_data: &Vec<InstanceRaw>) -> wgpu::Buffer {

     device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
}