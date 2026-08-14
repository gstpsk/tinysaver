use image::GenericImageView;

use crate::renderer::RenderContext;

pub fn create_diffuse_texture(ctx: &RenderContext, label: Option<&str>, image: image::DynamicImage) -> wgpu::Texture {
    let dimensions = image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        // All textures are stored as 3D, we represent our 2D texture
        // by setting depth to 1.
        depth_or_array_layers: 1,
    };

    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: label,
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &image.to_rgba8(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0), // RGBA * width
            rows_per_image: Some(dimensions.1), // height
        },
        texture_size,
    );

    texture
}

pub fn create_dummy_texture(ctx: &RenderContext) -> wgpu::Texture {
    let dummy_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Dummy texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let white_pixel: [u8; 4] = [255, 255, 255, 255];

    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &dummy_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &white_pixel,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );

    dummy_texture
}