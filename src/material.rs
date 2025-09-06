use raylib::prelude::{Color, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Vector3,
}

impl Material {
    pub fn new(diffuse: Vector3) -> Self {
        Material { diffuse }
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