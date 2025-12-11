/// Load image in texture in native mode . i.e non wasm32.

pub fn load_texture_from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    url: &str,
) -> anyhow::Result<wgpu::Texture> {

    // Image uses Rayon not available in Wasm
    let img = image::open(url)?.flipv().into_rgba8();

    let width = img.width();
    let height = img.height();

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

    queue.write_texture(
            // Tells wgpu where to copy the pixel data
    wgpu::TexelCopyTextureInfo {
        texture: &texture, // Destination
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
        },
        &img,                    // RGBA8 bytes
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(img.width() * 4),
            rows_per_image: Some(img.height()),
        },
        texture_size,
    );

    Ok(texture)
}
