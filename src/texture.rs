// texture.rs
use raylib::prelude::*;
use std::collections::HashMap;

pub struct Texture {
    pub width: i32,
    pub height: i32,
    pub data: Vec<Color>,
}

impl Texture {
    pub fn new(image: &Image) -> Self {
    Texture {
        width: image.width(),
        height: image.height(),
        // get_image_data() devuelve un slice, lo convertimos a un Vec
        data: image.get_image_data().to_vec(),
        }
    }
    
    pub fn get_color(&self, u: f32, v: f32) -> Color {
        let x = (u * (self.width - 1) as f32).round() as i32;
        let y = ((1.0 - v) * (self.height - 1) as f32).round() as i32; // Invertimos v porque las coordenadas de textura suelen empezar desde abajo

        let x = x.clamp(0, self.width - 1);
        let y = y.clamp(0, self.height - 1);

        self.data[(y * self.width + x) as usize]
    }
}

pub struct TextureManager {
    textures: HashMap<usize, Texture>,
    next_id: usize,
}

impl TextureManager {
    pub fn new() -> Self {
        TextureManager {
            textures: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn load_texture(&mut self, file_path: &str) -> Result<usize, String> {
        let image = match Image::load_image(file_path) {
            Ok(img) => img,
            Err(e) => return Err(e.to_string()),
        };

        let texture = Texture::new(&image);
        let id = self.next_id;
        self.textures.insert(id, texture);
        self.next_id += 1;
        Ok(id)
    }

    pub fn get_texture(&self, id: usize) -> Option<&Texture> {
        self.textures.get(&id)
    }
}