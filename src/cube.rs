// cube.rs
#[warn(unused_assignments)]

use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;
use raylib::prelude::{Vector2, Vector3}; // <-- Añade Vector2

pub struct Cube {
    pub center: Vector3,
    pub size: f32,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let half_size = self.size * 0.5;
        let min = self.center - Vector3::new(half_size, half_size, half_size);
        let max = self.center + Vector3::new(half_size, half_size, half_size);

        let mut tmin = (min.x - ray_origin.x) / ray_direction.x;
        let mut tmax = (max.x - ray_origin.x) / ray_direction.x;

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (min.y - ray_origin.y) / ray_direction.y;
        let mut tymax = (max.y - ray_origin.y) / ray_direction.y;

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return Intersect::empty();
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (min.z - ray_origin.z) / ray_direction.z;
        let mut tzmax = (max.z - ray_origin.z) / ray_direction.z;

        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return Intersect::empty();
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        // Comprobación final de tmax
        // if tzmax < tmax {
        //     tmax = tzmax;
        // }

        if tmin < 0.0 {
            return Intersect::empty();
        }

        let point = *ray_origin + *ray_direction * tmin;
        let p_local = point - self.center; // Punto relativo al centro del cubo

        let (normal, uv) = {
            let abs_p = Vector3::new(p_local.x.abs(), p_local.y.abs(), p_local.z.abs());
            
            if abs_p.x > abs_p.y && abs_p.x > abs_p.z { // Cara X
                let sign = p_local.x.signum();
                let n = Vector3::new(sign, 0.0, 0.0);
                let uv = Vector2::new(
                    (p_local.z * sign + half_size) / self.size,
                    (p_local.y + half_size) / self.size,
                );
                (n, uv)
            } else if abs_p.y > abs_p.z { // Cara Y
                let sign = p_local.y.signum();
                let n = Vector3::new(0.0, sign, 0.0);
                let uv = Vector2::new(
                    (p_local.x + half_size) / self.size,
                    (p_local.z * -sign + half_size) / self.size,
                );
                (n, uv)
            } else { // Cara Z
                let sign = p_local.z.signum();
                let n = Vector3::new(0.0, 0.0, sign);
                let uv = Vector2::new(
                    (p_local.x * -sign + half_size) / self.size,
                    (p_local.y + half_size) / self.size,
                );
                (n, uv)
            }
        };

        Intersect {
            distance: tmin,
            is_intersecting: true,
            point,
            normal,
            material: self.material,
            uv: Some(uv),
        }
    }
}