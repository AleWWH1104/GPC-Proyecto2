// ray_intersect.rs
use raylib::prelude::{Color, Vector2, Vector3}; // <-- Añade Vector2
use crate::material::Material;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Intersect {
    pub material: Material,
    pub distance: f32,
    pub is_intersecting: bool,
    pub normal: Vector3,
    pub point: Vector3,
    pub uv: Option<Vector2>, // <-- Añade esta línea
}

impl Intersect {
    pub fn new(material: Material, distance: f32, normal: Vector3, point: Vector3, uv: Option<Vector2>) -> Self {
        Intersect {
            material,
            distance,
            is_intersecting: true,
            normal,
            point,
            uv, // <-- Añade esta línea
        }
    }

    pub fn empty() -> Self {
        Intersect {
            material: Material::black(),
            distance: 0.0,
            is_intersecting: false,
            normal: Vector3::zero(),
            point: Vector3::zero(),
            uv: None, // <-- Añade esta línea
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect;
}