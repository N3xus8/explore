#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlImageElement;
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wgpu::ExternalImageSource;

cfg_if::cfg_if!{

 if #[cfg(target_arch = "wasm32")] {
        // 1. Helper to load the image async
        // We wrap the onload/onerror callbacks in a Promise
        fn load_image(url: &str) -> Promise {
            Promise::new(&mut |resolve, reject| {
                let img = HtmlImageElement::new().unwrap();
                img.set_cross_origin(Some("anonymous")); // Crucial for canvas security
                img.set_src(url);
                let value = img.clone();
                let onload = Closure::once(Box::new(move || {
                    resolve.call1(&JsValue::NULL, &value).unwrap();
                }));

                let onerror = Closure::once(Box::new(move || {
                    reject.call0(&JsValue::NULL).unwrap();
                }));

                img.set_onload(Some(onload.as_ref().unchecked_ref()));
                img.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                // Keep closures alive until callback fires
                onload.forget();
                onerror.forget();
            })
        }

        pub async fn load_texture_directly(
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            url: &str
        ) -> Result<wgpu::Texture, JsValue> {
            
            // 1. Load Image (Same helper as above)
            let promise = load_image(url);
            let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
            let img: HtmlImageElement = result.dyn_into()?;

            let width = img.width();
            let height = img.height();

            // 2. Create the WGPU Texture
            let texture_size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // TEXTURE_BINDING is required to use it in shaders
                // COPY_DST is required to copy data into it
                // RENDER_ATTACHMENT is required for copy_external_image_to_texture on some backends
                usage: wgpu::TextureUsages::TEXTURE_BINDING 
                    | wgpu::TextureUsages::COPY_DST 
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                view_formats: &[],
            });

            // 3. The Magic: Copy directly from DOM element to WGPU Texture
            let image_source = wgpu::CopyExternalImageSourceInfo {
                source: ExternalImageSource::HTMLImageElement(img),
                origin: wgpu::Origin2d::ZERO,
                flip_y: false, // Flip if your UVs require it
            };

            let image_destination = wgpu::CopyExternalImageDestInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                    color_space: wgpu::PredefinedColorSpace::Srgb,
                    premultiplied_alpha: false,
                };
            queue.copy_external_image_to_texture(
                &image_source,
                image_destination,
                texture_size,
            );

            Ok(texture)
        }
    }
}