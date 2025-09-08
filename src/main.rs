#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod cube;
mod camera;
mod material;
mod light; 
mod texture;

use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, Intersect};
use cube::Cube;
use camera::Camera;
use material::{Material, vector3_to_color, color_to_vector3};
use light::Light;
use texture::TextureManager;

fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    *incident - *normal * 2.0 * incident.dot(*normal)
}

fn cast_shadow<T: RayIntersect>(
    intersect: &Intersect,
    light: &Light,
    objects: &[T],
) -> f32 {
    let light_direction = (light.position - intersect.point).normalized();
    let shadow_ray_origin = intersect.point + intersect.normal * 1e-4;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_direction);
        if shadow_intersect.is_intersecting {
            return 0.8; //cambiar esto a una proporcion de la distancia para que haga el sh
        }
    }
    0.0
}

pub fn cast_ray<T: RayIntersect>(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[T],
    light: &Light,
    texture_manager: &TextureManager, // <-- Pasa el texture manager
) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting {
            if tmp.distance < zbuffer {
                zbuffer = tmp.distance;
                intersect = tmp;
            }
        }
    }

    if !intersect.is_intersecting {
        return Color::new(20, 20, 80, 255);
    }
    
    // --- NUEVA LÓGICA DE TEXTURA ---
    let mut diffuse_color = intersect.material.diffuse;
    if let Some(texture_id) = intersect.material.texture_id {
        if let Some(uv) = intersect.uv {
            if let Some(texture) = texture_manager.get_texture(texture_id) {
                let texture_color = texture.get_color(uv.x, uv.y);
                diffuse_color = color_to_vector3(texture_color);
            }
        }
    }
    // ----------------------------

    let light_direction = (light.position - intersect.point).normalized();
    let view_direction = (*ray_origin - intersect.point).normalized();
    let reflection_direction = reflect(&-light_direction, &intersect.normal).normalized();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);
    
    // Ambient
    let ambient_intensity = 0.2;
    let ambient = diffuse_color * ambient_intensity; // <-- Usa diffuse_color

    // Difuso
    let diffuse_intensity = intersect.normal.dot(light_direction).max(0.0) * light_intensity;
    let diffuse = diffuse_color * diffuse_intensity; // <-- Usa diffuse_color
    
    // Especular
    let specular_intensity = view_direction.dot(reflection_direction).max(0.0).powf(intersect.material.specular) * light_intensity;
    let specular = light.color * specular_intensity;
    
    // Color final
    let color = ambient + diffuse * intersect.material.albedo[0] + specular * intersect.material.albedo[1];

    vector3_to_color(color)
}

pub fn render<T: RayIntersect>(
    framebuffer: &mut Framebuffer, 
    objects: &[T], 
    camera: &Camera, 
    light: &Light,
    texture_manager: &TextureManager, // <-- Pasa el texture manager
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, texture_manager); // <-- Pasa el texture manager

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}
fn main() {
    let window_width = 1000;
    let window_height = 900;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer - Iris Ayala")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let background_color = Color::BLACK;
    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32, background_color);

    // --- Carga de textura ---
    let mut texture_manager = TextureManager::new();
    let cube_texture_id = texture_manager.load_texture("assets/marmol.jpg")
        .expect("No se pudo cargar la textura");
    // ------------------------

    let mut red_textured = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0), // El color base se multiplicará por la textura
        albedo: [0.9, 0.1], // Ajusta la influencia de difuso vs especular
        specular: 10.0,
        texture_id: Some(cube_texture_id), // <-- Asigna el ID de la textura
    };

    let objects = [
        Cube {
            center: Vector3::new(0.0, -0.4, -2.0),
            size: 2.0,
            material: red_textured,
        },
    ];

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;

    let light = Light::new(
        Vector3::new(0.0, 2.0, 4.0),
        Vector3::new(1.0, 1.0, 1.0),
        2.5,
    );

    while !window.window_should_close() {
        framebuffer.clear();

        // ... controles de cámara ...
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, rotation_speed);
        }

        render(&mut framebuffer, &objects, &camera, &light, &texture_manager); // <-- Pasa el texture manager

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}