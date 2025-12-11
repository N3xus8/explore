use anyhow::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;
#[cfg(not(target_arch = "wasm32"))] 
use crate::utils::load_texture_from_image;
#[cfg(target_arch = "wasm32")] 
use crate::web_utils::load_texture_from_image_web;

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {

    pub async fn get_texture_from_image(
                device: &wgpu::Device,
                queue: &wgpu::Queue, 
                url: &str
        ) -> Result<Self> {


            #[cfg(target_arch = "wasm32")]
            let texture =  load_texture_from_image_web(device, queue, url).await.map_err(|e| log::error!("texture error {:?} ", e)).unwrap_throw();
               
            
           
            #[cfg(not(target_arch = "wasm32"))]
            let texture = load_texture_from_image( &device, &queue,url)?;
      
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
                }
            );



              Ok(Self { texture, view, sampler })

    }

    pub fn bind_group_for_texture(&self, 
                device: &wgpu::Device,
        ) -> (wgpu::BindGroupLayout, wgpu::BindGroup ){

        let texture_bind_group_layout  =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });


        let diffuse_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view), 
                 },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler), 
                }
            ],
            label: Some("diffuse_bind_group"),
            }
        );

        (texture_bind_group_layout,diffuse_bind_group)

    }

}