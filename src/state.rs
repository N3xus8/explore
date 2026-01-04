use std::sync::Arc;

use cgmath::Vector3;
use instant::Instant;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{
    camera::{
        bind_group_for_camera_uniform, create_camera_buffer, create_camera_reflected_buffer,
        Camera, CameraController, CameraUniform,
    },
    depth_stencil::{self, StencilTexture},
    extra::{MirrorPlaneUniform, Spin, SpinUniform},
    model::{DrawModel, Model},
    pipeline::Pipeline,
    resources,
    texture::{self, create_multisampled_view, Texture},
    utils::build_reflection_matrix,
    vertex::{
        create_index_buffer, create_instance_buffer, create_vertex_buffer, Instance, INDICES,
        VERTICES,
    },
};

pub struct State {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    #[allow(dead_code)]
    pub diffuse_texture: Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub another_texture: Texture,
    pub another_bind_group: wgpu::BindGroup,
    is_space_pressed: bool,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    //depth_texture: Texture,
    obj_model: Model,
    last_frame: Instant,
    spin: Spin,
    spin_uniform: SpinUniform,
    spin_buffer: wgpu::Buffer,
    spin_bind_group: wgpu::BindGroup,
    depth_stencil: StencilTexture,
    stencil_pipeline: wgpu::RenderPipeline,
    reflection_pipeline: wgpu::RenderPipeline,
    mirror_instance: Instance,
    mirror_instance_buffer: wgpu::Buffer,
    mirror_plane_uniform: MirrorPlaneUniform,
    mirror_plane_buffer: wgpu::Buffer,
    mirror_plane_bind_group: wgpu::BindGroup,
    //    camera_reflected_uniform: CameraUniform,
    camera_reflected_buffer: wgpu::Buffer,
    camera_reflected_bind_group: wgpu::BindGroup,
    mirror_surface_pipeline: wgpu::RenderPipeline,
    multisampled_framebuffer: Option<wgpu::TextureView>,
    sample_count: u32,
    debug_stencil_pipeline: wgpu::RenderPipeline, // DEBUG
    pub window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // Surface
        let surface = instance.create_surface(window.clone())?;

        // Adapter

        let adapter = if cfg!(target_arch = "wasm32") {
            instance //  TODO select the most relevant adapter
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await?
        } else {
            instance // TODO select the most relevant adapter
                .enumerate_adapters(wgpu::Backends::all())
                .into_iter()
                .find(|adapter| adapter.is_surface_supported(&surface))
                .ok_or(anyhow::anyhow!("No adapter found"))?
        };

        // Device & Queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Vertex
        let vertex_buffer = create_vertex_buffer(&device)?;

        let num_vertices = VERTICES.len() as u32;

        // Indices

        let index_buffer = create_index_buffer(&device)?;
        let num_indices = INDICES.len() as u32;

        // Texture from Image

        let sample_count: u32 = 4;
        //let url = "images/github-colored-logo.png";
        let url = "images/wgpu-logo.png";
        let diffuse_texture =
            crate::texture::Texture::get_texture_from_image(&device, &queue, url).await?;

        let (diffuse_bind_group_layout, diffuse_bind_group) =
            diffuse_texture.bind_group_for_texture(&device);

        let another_url = "images/github-icon-logo.png";
        let another_texture =
            crate::texture::Texture::get_texture_from_image(&device, &queue, another_url).await?;
        let (_another_bind_group_layout, another_bind_group) =
            another_texture.bind_group_for_texture(&device);

        let obj_model = resources::load_model(
            "models/cube.obj",
            &device,
            &queue,
            &diffuse_bind_group_layout,
        )
        .await
        .unwrap();

        // /
        // / I N S T A N C E S
        // /

        let instances = Instance::generate_instances();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = create_instance_buffer(&device, &instance_data);

        let mirror_instance = Instance::generate_instance(5.0, 1.0, 2.0, 45.0);
        let mirror_instance_data = mirror_instance.to_raw_with_scale(1.5);
        let mirror_instance_buffer = create_instance_buffer(&device, &[mirror_instance_data]);

        // / M I R R O R  P L A N E  U N I F O R M

        let mirror_plane_uniform =
            MirrorPlaneUniform::new(&mirror_instance.transform(), Vector3::unit_z());
        let mirror_plane_buffer = mirror_plane_uniform.mirror_plane_buffer(&device);
        let (mirror_plane_bind_group_layout, mirror_plane_bind_group) =
            MirrorPlaneUniform::create_bind_group_layout(&device, &mirror_plane_buffer);

        // / C A M E R A
        // /
        let camera = Camera::new(
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            (0.0, 1.0, 2.0),
            // have it look at the origin
            (0.0, 0.0, 0.0),
            // which way is "up"
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = create_camera_buffer(&camera_uniform, &device);
        let (camera_bind_group_layout, camera_bind_group) =
            bind_group_for_camera_uniform(&camera_buffer, &device);

        let camera_controller = CameraController::new(0.1);

        let camera_reflected_uniform = CameraUniform::new();
        //camera_reflected_uniform.update_view_proj(&camera);
        let camera_reflected_buffer =
            create_camera_reflected_buffer(&camera_reflected_uniform, &device);
        let (camera_reflected_bind_group_layout, camera_reflected_bind_group) =
            bind_group_for_camera_uniform(&camera_reflected_buffer, &device);

        // / D E P T H   T E X T U R E
        // /

        // let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // / S T E N C I L  T E X T U R E
        // /

        let depth_stencil = depth_stencil::StencilTexture::create_stencil_texture(
            &device,
            &config,
            "depth_stencil",
            sample_count,
        );

        // / S P I N

        let last_frame = Instant::now();
        let spin = Spin::new(1.5);
        let spin_uniform = SpinUniform::new();
        let spin_buffer = spin_uniform.create_spin_uniform_buffer(&device);
        let (spin_bind_group_layout, spin_bind_group) =
            SpinUniform::bind_group_for_spin_uniform(&spin_buffer, &device);

        // /
        // / MultiSample Framebuffer

        let multisampled_framebuffer: Option<wgpu::TextureView> = if sample_count > 1 {
            Some(texture::create_multisampled_view(
                &device,
                &config,
                sample_count,
            ))
        } else {
            None
        };

        // / P I P E L I N E S
        // /

        // Pipeline
        let pipeline_struct = Pipeline::build_render_pipeline(
            &device,
            &config,
            sample_count,
            &diffuse_bind_group_layout,
            &camera_bind_group_layout,
            &spin_bind_group_layout,
            &mirror_plane_bind_group_layout,
        )?;
        let render_pipeline = pipeline_struct.pipeline;

        // Stencil Pipeline
        let stencil_pipeline_struct =
            Pipeline::mask_render_pipeline(&device, &camera_bind_group_layout, sample_count)?;

        let stencil_pipeline = stencil_pipeline_struct.pipeline;

        // Reflection Pipeline

        let reflection_pipeline_struct = Pipeline::reflection_render_pipeline(
            &device,
            &config,
            &diffuse_bind_group_layout,
            &camera_reflected_bind_group_layout,
            &spin_bind_group_layout,
            &mirror_plane_bind_group_layout,
            sample_count,
        )?;

        let reflection_pipeline = reflection_pipeline_struct.pipeline;

        // Mirror surface pipeline

        let mirror_surface_pipeline_struct = Pipeline::mirror_surface_render_pipeline(
            &device,
            &config,
            &camera_bind_group_layout,
            sample_count,
        )?;

        let mirror_surface_pipeline = mirror_surface_pipeline_struct.pipeline;

        // ================================
        // /  D E B U G G I N G
        use crate::debugger::Pipeline as DebugPipeline;
        let debug_stencil_pipeline_struct =
            DebugPipeline::debug_render_pipeline(&device, &config, sample_count)?;
        let debug_stencil_pipeline = debug_stencil_pipeline_struct.pipeline;

        // ================================

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            num_indices,
            index_buffer,
            diffuse_texture,
            diffuse_bind_group,
            another_texture,
            another_bind_group,
            is_space_pressed: false,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instances,
            instance_buffer,
            //   depth_texture,
            obj_model,
            last_frame,
            spin_uniform,
            spin_buffer,
            spin_bind_group,
            spin,
            depth_stencil,
            stencil_pipeline,
            reflection_pipeline,
            mirror_instance,
            mirror_instance_buffer,
            mirror_plane_uniform,
            mirror_plane_buffer,
            mirror_plane_bind_group,
            //            camera_reflected_uniform,
            camera_reflected_bind_group,
            camera_reflected_buffer,
            mirror_surface_pipeline,
            multisampled_framebuffer,
            sample_count,
            debug_stencil_pipeline, // DEBUG
            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.camera.aspect = width as f32 / height as f32;

            // This is a fix from chatgpt otherwise it only works for desktop not for browser.
            self.camera_uniform.update_view_proj(&self.camera);
            //self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.depth_stencil = depth_stencil::StencilTexture::create_stencil_texture(
                &self.device,
                &self.config,
                "depth_stencil",
                self.sample_count,
            );

            if self.multisampled_framebuffer.is_some() {
                self.multisampled_framebuffer = Some(create_multisampled_view(
                    &self.device,
                    &self.config,
                    self.sample_count,
                ));
            };
        }
    }

    pub fn update(&mut self) {
        // Delta time
        let now = Instant::now();
        let mut dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        // Clamp for browser tab resume
        dt = dt.min(0.1);

        // Update logic
        self.spin.update(dt);

        // Update GPU data
        self.spin_uniform.update_from_angle(self.spin.angle());

        self.queue.write_buffer(
            &self.spin_buffer,
            0,
            bytemuck::bytes_of(&[self.spin_uniform]),
        );

        // Camera
        self.camera_controller.update_camera(&mut self.camera);
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Camera uniform normal mode (for non-reflected mode)

        self.camera_uniform.update_view_proj(&self.camera);

        //
        // S T E N C I L   P A S S
        //

        // Write Camera buffer

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        // /

        let mut stencil_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stencil pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_stencil.view,
                depth_ops: None,
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: wgpu::StoreOp::Store,
                }),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        stencil_pass.set_stencil_reference(1);
        stencil_pass.set_pipeline(&self.stencil_pipeline);
        stencil_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        stencil_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        stencil_pass.set_vertex_buffer(1, self.mirror_instance_buffer.slice(..));
        stencil_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //stencil_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        //stencil_pass.draw_indexed(0..self.num_indices, 0, 0..self.mirror_instance.len() as _);
        stencil_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(stencil_pass);

        // / =================================
        // /    D E B U G G I N G
        // /

        let render_pass_color_attachments = match &self.multisampled_framebuffer {
            Some(texture_view) => wgpu::RenderPassColorAttachment {
                view: texture_view,
                depth_slice: None,
                resolve_target: Some(&view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            },
            None => wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            },
        };

        let mut debug_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Stencil Debug Pass"),
            color_attachments: &[Some(render_pass_color_attachments)],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_stencil.view,
                depth_ops: None,
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        debug_pass.set_pipeline(&self.debug_stencil_pipeline);
        debug_pass.set_stencil_reference(1);
        debug_pass.draw(0..3, 0..1);
        drop(debug_pass);

        // // // ============================

        //
        // /  R E F L E C T I O N   P A S S
        //

        //  I N   F L U X

        // Write Camera buffer
        // 1. Get mirror plane from CPU-side transform

        let mirror_transform = Instance::transform(&self.mirror_instance);

        let reflection = build_reflection_matrix(&mirror_transform, Vector3::unit_z());

        let reflected_camera: [[f32; 4]; 4] = self.camera.build_reflected_camera(reflection).into();

        self.queue.write_buffer(
            &self.camera_reflected_buffer,
            0,
            bytemuck::cast_slice(&[reflected_camera]),
        );
        self.queue.write_buffer(
            &self.mirror_plane_buffer,
            0,
            bytemuck::cast_slice(&[self.mirror_plane_uniform]),
        );
        // /

        let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_stencil.view,

            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0), // <- clear depth
                store: wgpu::StoreOp::Store,
            }),

            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load, // <- keep mirror mask
                store: wgpu::StoreOp::Store,
            }),
        };

        let render_pass_color_attachments = match &self.multisampled_framebuffer {
            Some(texture_view) => wgpu::RenderPassColorAttachment {
                view: texture_view,
                depth_slice: None,
                resolve_target: Some(&view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            },
            None => wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            },
        };
        let mut reflection_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("reflection pass"),
            color_attachments: &[Some(render_pass_color_attachments)],
            depth_stencil_attachment: Some(depth_stencil_attachment),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        reflection_pass.set_pipeline(&self.reflection_pipeline);
        reflection_pass.set_stencil_reference(1);
        reflection_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        reflection_pass.set_bind_group(1, &self.camera_reflected_bind_group, &[]);
        reflection_pass.set_bind_group(2, &self.spin_bind_group, &[]);
        reflection_pass.set_bind_group(3, &self.mirror_plane_bind_group, &[]);
        reflection_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        reflection_pass
            .draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);

        drop(reflection_pass);

        // /
        // T O T A L  S C E N E
        // /

        // Write Camera buffer
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.queue.write_buffer(
            &self.mirror_plane_buffer,
            0,
            bytemuck::cast_slice(&[self.mirror_plane_uniform]),
        );

        //
        let render_pass_color_attachments = match &self.multisampled_framebuffer {
            Some(texture_view) => {
                wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    depth_slice: None,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // <- DO NOT CLEAR
                        store: wgpu::StoreOp::Store,
                    },
                }
            }
            None => wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // <- DO NOT CLEAR
                    store: wgpu::StoreOp::Store,
                },
            },
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Total Scene Pass"),
            color_attachments: &[Some(render_pass_color_attachments)],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_stencil.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load, // <- clear depth again
                    store: wgpu::StoreOp::Store,
                }),
                // stencil_ops: Some(wgpu::Operations {
                //     load: wgpu::LoadOp::Load,
                //     store: wgpu::StoreOp::Store,
                // }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let bind_group = if self.is_space_pressed {
            &self.another_bind_group
        } else {
            &self.diffuse_bind_group
        };

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(2, &self.spin_bind_group, &[]);
        render_pass.set_bind_group(3, &self.mirror_plane_bind_group, &[]);
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        //render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        //render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

        render_pass.draw_mesh_instanced(&self.obj_model.meshes[0], 0..self.instances.len() as u32);
        // TODO MIRROR
        drop(render_pass);

        // / M I R R O R   S U R F A C E
        // /
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let render_pass_color_attachments = match &self.multisampled_framebuffer {
            Some(texture_view) => wgpu::RenderPassColorAttachment {
                view: texture_view,
                depth_slice: None,
                resolve_target: Some(&view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // <- DO NOT CLEAR
                    store: wgpu::StoreOp::Store,
                },
            },
            None => wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // <- DO NOT CLEAR
                    store: wgpu::StoreOp::Store,
                },
            },
        };

        let mut mirror_surface_render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mirror surface Render Pass"),
                color_attachments: &[Some(render_pass_color_attachments)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load, // <- clear depth again
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

        mirror_surface_render_pass.set_pipeline(&self.mirror_surface_pipeline);
        mirror_surface_render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        mirror_surface_render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        mirror_surface_render_pass.set_vertex_buffer(1, self.mirror_instance_buffer.slice(..));
        mirror_surface_render_pass
            .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        mirror_surface_render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(mirror_surface_render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Space, is_pressed) => self.is_space_pressed = is_pressed,
            (
                KeyCode::KeyW
                | KeyCode::ArrowUp
                | KeyCode::KeyA
                | KeyCode::ArrowLeft
                | KeyCode::KeyS
                | KeyCode::ArrowDown
                | KeyCode::KeyD
                | KeyCode::ArrowRight,
                is_pressed,
            ) => self.camera_controller.handle_key(code, is_pressed),
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}
