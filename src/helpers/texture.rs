use image::{GenericImageView, ImageReader};
use std::io::Cursor;
use wgpu::{
    Device, Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

/// Texture type describing how the texture is used in the shader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureType {
    /// 2D texture (standard diffuse, normal map, etc.)
    Texture2D,
    /// Cubemap texture (6 faces)
    TextureCube,
}

/// Configuration for creating a texture
pub struct TextureConfig {
    pub texture_type: TextureType,
    pub format: TextureFormat,
    pub label: Option<&'static str>,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            texture_type: TextureType::Texture2D,
            format: TextureFormat::Rgba8UnormSrgb,
            label: Some("texture"),
        }
    }
}

/// Load a single 2D texture from bytes
pub fn load_texture_2d(
    device: &Device,
    queue: &Queue,
    bytes: &[u8],
    config: TextureConfig,
) -> Result<Texture, String> {
    if config.texture_type != TextureType::Texture2D {
        return Err("TextureConfig must be Texture2D for this function".to_string());
    }

    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let rgba = image.to_rgba8();
    let dimensions = image.dimensions();

    let size = Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: config.format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        label: config.label,
        view_formats: &[],
    });

    queue.write_texture(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        size,
    );

    Ok(texture)
}

/// Load a cubemap texture from 6 individual images (right, left, top, bottom, front, back)
pub fn load_texture_cubemap(
    device: &Device,
    queue: &Queue,
    bytes_array: [&[u8]; 6],
    config: TextureConfig,
) -> Result<Texture, String> {
    if config.texture_type != TextureType::TextureCube {
        return Err("TextureConfig must be TextureCube for this function".to_string());
    }

    let mut images = Vec::new();
    let mut dimensions = (0, 0);

    // Load all 6 images
    for (i, bytes) in bytes_array.iter().enumerate() {
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| format!("Failed to guess format: {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image {}: {}", i, e))?;

        let dims = image.dimensions();
        if i == 0 {
            dimensions = dims;
        } else if dims != dimensions {
            return Err(format!(
                "All cubemap faces must be the same size. Expected {:?}, got {:?}",
                dimensions, dims
            ));
        }

        images.push(image.to_rgba8());
    }

    let size = Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 6,
    };

    let texture = device.create_texture(&TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: config.format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        label: config.label,
        view_formats: &[],
    });

    // Write each face
    for (face_index, rgba_data) in images.iter().enumerate() {
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d {
                    x: 0,
                    y: 0,
                    z: face_index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );
    }

    Ok(texture)
}

/// Update an existing 2D texture with new image data
pub fn update_texture_2d(queue: &Queue, texture: &Texture, bytes: &[u8]) -> Result<(), String> {
    let image = ImageReader::new(Cursor::new(bytes))
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let rgba = image.to_rgba8();
    let dimensions = image.dimensions();

    let size = Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        size,
    );

    Ok(())
}
