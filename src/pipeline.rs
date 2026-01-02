
use anyhow::*;

use crate::{model::{ModelVertex, Vertex}, vertex::InstanceRaw};
use crate::vertex::Vertex as PrimitiveVertex;
pub struct Pipeline {
   pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn build_render_pipeline(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                texture_bind_group_layout: &wgpu::BindGroupLayout,
                camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
                spin_uniform_bind_group_layout: &wgpu::BindGroupLayout,
                mirror_plane_uniform_bind_group_layout: &wgpu::BindGroupLayout, // TODO REVIEW currently not used. Is this needed. test with object behind mirror
        ) -> Result<Pipeline> {


            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout, camera_uniform_bind_group_layout, spin_uniform_bind_group_layout, mirror_plane_uniform_bind_group_layout],                
                push_constant_ranges: &[],
            });

            //Pipeline
            
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"), // 1.
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()], // 2.
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState { // 3.
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState { // 4.
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less, 
                        //stencil: wgpu::StencilState::default(), 
                        stencil: wgpu::StencilState {
                            front: wgpu::StencilFaceState {
                                compare: wgpu::CompareFunction::Less,
                                fail_op: wgpu::StencilOperation::Keep,
                                depth_fail_op: wgpu::StencilOperation::Keep,
                                pass_op: wgpu::StencilOperation::Keep,
                            },
                            back: wgpu::StencilFaceState::IGNORE,
                            read_mask: 0xFF,
                            write_mask: 0x00,
                        }, 

                        bias: wgpu::DepthBiasState::default(),
                    }), 
                    multisample: wgpu::MultisampleState {
                        count: 1, // 2.
                        mask: !0, // 3.
                        alpha_to_coverage_enabled: false, // 4.
                    },
                    multiview: None, // 5.
                    cache: None, // 6.
            });
        Ok(Self {pipeline: render_pipeline})
    }

    pub fn mask_render_pipeline(
            device: &wgpu::Device,
            camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    )  -> Result<Pipeline> {

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("stencil"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/stencil.wgsl").into()),
            });

        let mask_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("mask_pipeline_layout"),
                bind_group_layouts: &[camera_uniform_bind_group_layout,], 
                push_constant_ranges: &[],
            });

        let mask_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Mask_Render_Pipeline"),
                layout: Some(&mask_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"), // 1.
                    buffers: &[PrimitiveVertex::desc(), InstanceRaw::desc()], // 2.
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: None,
                primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Always,
                        stencil: wgpu::StencilState {
                            front: wgpu::StencilFaceState {
                                compare: wgpu::CompareFunction::Always,
                                fail_op: wgpu::StencilOperation::Keep,
                                depth_fail_op: wgpu::StencilOperation::Keep,
                                pass_op: wgpu::StencilOperation::Replace,
                            },
                            back: wgpu::StencilFaceState::IGNORE,
                            read_mask: 0xFF,
                            write_mask: 0xFF,
                        },
                        bias: wgpu::DepthBiasState::default(),
                    }), 
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None, 
                    cache: None, 
            });

            Ok(Self {pipeline: mask_render_pipeline})
    }

    pub fn reflection_render_pipeline(
            device: &wgpu::Device,
            config: &wgpu::SurfaceConfiguration,
            texture_bind_group_layout: &wgpu::BindGroupLayout,
            camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
            spin_uniform_bind_group_layout: &wgpu::BindGroupLayout,
            mirror_plane_uniform_bind_group_layout: &wgpu::BindGroupLayout,

    )  -> Result<Pipeline> {

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("mirror reflection"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mirror_reflection.wgsl").into()),
            });
        
        let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[texture_bind_group_layout, camera_uniform_bind_group_layout, spin_uniform_bind_group_layout, mirror_plane_uniform_bind_group_layout],                
                push_constant_ranges: &[],
            });

        let reflected_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Reflected_scene_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex:  wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"), // 1.
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()], // 2.
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
            fragment: Some(wgpu::FragmentState { // 3.
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState { // 4.
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                        pass_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0xFF,
                    write_mask: 0x00,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw, //  TODO: Check and tune
                cull_mode: None, // important: reflection flips winding
                ..Default::default()
            },
            multiview: None, 
            cache: None, 
            multisample: wgpu::MultisampleState::default(),
        });

        Ok(Self {pipeline: reflected_pipeline})

    }

    pub fn mirror_surface_render_pipeline(
                device: &wgpu::Device,
                config: &wgpu::SurfaceConfiguration,
                camera_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        ) -> Result<Pipeline> {


            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Mirror surface"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mirror_surface.wgsl").into()),
            });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mirror surface Pipeline Layout"),
                bind_group_layouts: &[camera_uniform_bind_group_layout],                
                push_constant_ranges: &[],
            });

            //Pipeline
            
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Mirror surface Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"), // 1.
                    buffers: &[PrimitiveVertex::desc(), InstanceRaw::desc()], // 2.
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState { // 3.
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState { // 4.
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    //cull_mode: Some(wgpu::Face::Back),
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Less, 
                        //stencil: wgpu::StencilState::default(), 
                        stencil: wgpu::StencilState {
                            front: wgpu::StencilFaceState {
                                compare: wgpu::CompareFunction::Less,
                                fail_op: wgpu::StencilOperation::Keep,
                                depth_fail_op: wgpu::StencilOperation::Keep,
                                pass_op: wgpu::StencilOperation::Keep,
                            },
                            back: wgpu::StencilFaceState::IGNORE,
                            read_mask: 0xFF,
                            write_mask: 0x00,
                        }, 

                        bias: wgpu::DepthBiasState::default(),
                    }), 
                    multisample: wgpu::MultisampleState {
                        count: 1, // 2.
                        mask: !0, // 3.
                        alpha_to_coverage_enabled: false, // 4.
                    },
                    multiview: None, // 5.
                    cache: None, // 6.
            });
        Ok(Self {pipeline: render_pipeline})
    }

}