// material.rs
use raylib::prelude::{Color, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Vector3,
    pub albedo: [f32; 2],
    pub specular: f32,
    pub texture_id: Option<usize>, // <-- Añade esta línea
}

impl Material {
    pub fn new(diffuse: Vector3, albedo: [f32; 2], specular: f32) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
            texture_id: None, // <-- Añade esta línea
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Vector3::zero(),
            albedo: [0.0, 0.0],
            specular: 0.0,
            texture_id: None, // <-- Añade esta línea
        }
    }
}

pub fn vector3_to_color(v: Vector3) -> Color {
    Color::new(
        (v.x * 255.0).min(255.0) as u8,
        (v.y * 255.0).min(255.0) as u8,
        (v.z * 255.0).min(255.0) as u8,
        255,
    )
}

pub fn color_to_vector3(c: Color) -> Vector3 {
    Vector3::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
    )
}