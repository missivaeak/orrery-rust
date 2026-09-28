use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, Queue, Texture};

use crate::helpers::texture::{TextureConfig, TextureType, load_texture_2d, load_texture_cubemap};

pub struct AssetLibrary {
    textures_2d: HashMap<String, Arc<Texture>>,
    textures_cubemap: HashMap<String, Arc<Texture>>,
    texture_types: HashMap<String, TextureType>,
}

impl AssetLibrary {
    pub fn new(device: &Device, queue: &Queue) -> Result<Self, String> {
        let mut textures_2d = HashMap::new();
        let mut textures_cubemap = HashMap::new();
        let mut texture_types = HashMap::new();

        // Load all 2D textures
        let earth_diffuse = load_texture_2d(
            device,
            queue,
            include_bytes!("../assets/earth/side_0.jpg"),
            TextureConfig::default(),
        )?;
        textures_2d.insert("earth_diffuse".to_string(), Arc::new(earth_diffuse));
        texture_types.insert("earth_diffuse".to_string(), TextureType::Texture2D);

        // Load all cubemaps (placeholder - you'll need actual cubemap images)
        // For now, we'll skip cubemap loading until you have the images
        let skybox = load_texture_cubemap(
            device,
            queue,
            [
                include_bytes!("../assets/earth/side_0.jpg"),
                include_bytes!("../assets/earth/side_1.jpg"),
                include_bytes!("../assets/earth/side_2.jpg"),
                include_bytes!("../assets/earth/side_3.jpg"),
                include_bytes!("../assets/earth/side_4.jpg"),
                include_bytes!("../assets/earth/side_5.jpg"),
            ],
            TextureConfig {
                texture_type: TextureType::TextureCube,
                ..Default::default()
            },
        )?;
        textures_cubemap.insert("earth_height".to_string(), Arc::new(skybox));
        texture_types.insert("earth_height".to_string(), TextureType::TextureCube);

        Ok(Self {
            textures_2d,
            textures_cubemap,
            texture_types,
        })
    }

    pub fn get_texture(&self, key: &str) -> Option<Arc<Texture>> {
        self.textures_2d
            .get(key)
            .or_else(|| self.textures_cubemap.get(key))
            .cloned()
    }

    pub fn get_texture_type(&self, key: &str) -> Option<TextureType> {
        self.texture_types.get(key).copied()
    }

    // pub fn texture_exists(&self, key: &str) -> bool {
    //     self.textures_2d.contains_key(key) || self.textures_cubemap.contains_key(key)
    // }
}
