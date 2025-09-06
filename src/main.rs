use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod sphere;
mod camera;
mod material;
mod light;

use framebuffer::Framebuffer;
use ray_intersect::{Intersect, RayIntersect};
use sphere::Sphere;
use camera::Camera;
use material::{Material, vector3_to_color};
use light::Light;

fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    *incident - *normal * 2.0 * incident.dot(*normal)
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Sphere],
    light: &Light,
) -> Color {
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = intersect.distance;
            intersect = tmp;
        }
    }

    if !intersect.is_intersecting {
        return Color::new(4, 12, 36, 255);
    }

    let light_dir = (light.position - intersect.point).normalized();
    let view_dir = (*ray_origin - intersect.point).normalized();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0) * light.intensity;
    let diffuse = intersect.material.diffuse * diffuse_intensity;

    let specular_intensity = view_dir.dot(reflect_dir).max(0.0).powf(intersect.material.specular) * light.intensity;
    let light_color_v3 = Vector3::new(light.color.r as f32 / 255.0, light.color.g as f32 / 255.0, light.color.b as f32 / 255.0);
    let specular = light_color_v3 * specular_intensity;
    
    let albedo = intersect.material.albedo;
    let final_color_v3 = diffuse * albedo[0] + specular * albedo[1];

    vector3_to_color(intersect.material.diffuse)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera,light: &Light) {
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

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
 
    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer Example")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let background_color = Color::BLACK;
    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32, background_color);

    let rubber = Material::new(
        Vector3::new(0.3, 0.1, 0.1),
        10.0,
        [0.9, 0.1],
    );

    let ivory = Material::new(
        Vector3::new(0.4, 0.4, 0.3),
        50.0,
        [0.6, 0.3],
    );

    let objects = [
        Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: ivory,
        },
        Sphere {
            center: Vector3::new(2.0, 0.0, -5.0),
            radius: 0.5,
            material: rubber,
        },
    ];

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;

    let light = Light::new(
        Vector3::new(5.0, 5.0, 5.0),
        Color::new(255, 255, 255, 255),
        1.5,
    );

    while !window.window_should_close() {
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

        framebuffer.clear();

        render(&mut framebuffer, &objects, &camera, &light);

        framebuffer.swap_buffers(&mut window, &thread);

    }
}