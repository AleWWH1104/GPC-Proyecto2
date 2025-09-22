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
        return Color::new(204, 184, 204, 255);
    }
    
    // --- NUEVA LÓGICA DE TEXTURA ---
    let mut diffuse_color = intersect.material.diffuse;
    if let Some(texture_id) = intersect.material.texture_id {
        if let Some(uv) = intersect.uv {
            if let Some(texture) = texture_manager.get_texture(texture_id) {
                diffuse_color = color_to_vector3(texture.get_color(uv.x, uv.y));
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

    // Carga de texturas
    let mut texture_manager = TextureManager::new();
    let wood_texture   = texture_manager.load_texture("assets/wood.png").unwrap();
    let grass_texture  = texture_manager.load_texture("assets/grass.png").unwrap();
    let water_texture  = texture_manager.load_texture("assets/water.png").unwrap();
    let leaves_texture = texture_manager.load_texture("assets/pink_leaves.png").unwrap();

    let MAT_WOOD = Material::new(
        Vector3::new(0.55, 0.38, 0.25),    // diffuse (marrón cálido)
        [0.95, 0.05],                      // albedo: difuso alto, especular bajo
        12.0                               // specular: highlight suave
    )
    .with_texture(wood_texture)
    .with_optics(0.0, 0.05, 1.0);          // transp=0, reflect=0.05, ior=1.0

    // Césped (terreno): muy difuso, rugoso
    let MAT_GRASS = Material::new(
        Vector3::new(0.35, 0.62, 0.22),    // verde vivo
        [0.97, 0.03],
        8.0
    )
    .with_texture(grass_texture)
    .with_optics(0.0, 0.03, 1.0);

    // Agua: con transparencia moderada y algo de reflejo (luego animamos UV)
    let MAT_WATER = Material::new(
        Vector3::new(0.12, 0.35, 0.65),    // azul con un poco de verde
        [0.80, 0.20],                      // difuso menor, algo de especular
        64.0                               // highlight más duro
    )
    .with_texture(water_texture)
    .with_optics(0.55, 0.20, 1.33);        // transp=0.55, reflect=0.20, ior=1.33

    // Hojas rosadas (copa): difusas, un pelín translúcidas
    let MAT_LEAVES = Material::new(
        Vector3::new(0.95, 0.55, 0.70),    // rosa cerezo
        [0.95, 0.05],
        10.0
    )
    .with_texture(leaves_texture)
    .with_optics(0.12, 0.02, 1.10); 

    let mut objects: Vec<Cube> = Vec::new();
    fn add(objects: &mut Vec<Cube>, x: f32, y: f32, z: f32, s: f32, m: Material) {
        objects.push(Cube { center: Vector3::new(x, y, z), size: s, material: m });
    }

    let mut add_cube = |x: f32, y: f32, z: f32, s: f32, m: Material| {
        add(&mut objects, x, y, z, s, m);
    };

    // Tamaño de voxel y alturas
    let tile: f32 = 1.0_f32;
    let y_floor0: f32 = -tile * 0.5_f32;           // cara superior del piso 0 queda en y=0
    let y_floor1: f32 = y_floor0 + tile;           // segundo piso

    // Duplicamos césped para que el oscuro sea "más alto" (mismo tex, color más oscuro)
    let mut MAT_GRASS_DARK = MAT_GRASS;
    MAT_GRASS_DARK.diffuse = Vector3::new(0.16, 0.35, 0.16);

    // B=agua, g=cesped claro (piso0), G=cesped oscuro (piso1), T=tronco (piso1 + tronco)
    const COLS: usize = 6;
    const ROWS: usize = 8;
    let grid: [[char; COLS]; ROWS] = [
        ['G','G','B','t','G','G'],
        ['G','G','B','g','G','G'],
        ['G','G','B','g','g','G'],
        ['G','G','B','B','g','T'],
        ['G','T','B','B','g','G'],
        ['G','G','g','B','g','g'],
        ['G','G','g','B','B','g'],
        ['t','g','g','B','B','g'],
    ];

    // Centramos el grid alrededor del origen
    let x0 = -((COLS as f32 - 1.0_f32) * 0.5_f32) * tile;
    let z0 = -((ROWS as f32 - 1.0_f32) * 0.5_f32) * tile;

    for r in 0..ROWS {
        for c in 0..COLS {
            let ch = grid[r][c];
            let x = x0 + c as f32 * tile;
            let z = z0 + r as f32 * tile;

            match ch {
                'B' => {
                    // agua en piso 0
                    add_cube(x, y_floor0, z, tile, MAT_WATER);
                }
                'g' => {
                    // césped claro en piso 0
                    add_cube(x, y_floor0, z, tile, MAT_GRASS);
                }
                'G' => {
                    // césped oscuro en piso 1 
                    add_cube(x, y_floor1, z, tile, MAT_GRASS_DARK);
                    //piso 0
                    add_cube(x, y_floor0, z, tile, MAT_GRASS);
                }
                't' => {
                    // tronco sobre piso 1
                    add_cube(x, y_floor0, z, tile, MAT_GRASS); // base opcional

                    // tronco de 2 cubos
                    let trunk_size: f32 = 0.9_f32;
                    let y_trunk0 = y_floor0 + trunk_size * 0.5_f32;
                    let y_trunk1 = y_trunk0 + trunk_size;
                    add_cube(x, y_trunk0, z, trunk_size, MAT_WOOD);
                    add_cube(x, y_trunk1, z, trunk_size, MAT_WOOD);

                    // Copa: 3 niveles 
                    let leaf_size: f32 = 1.0_f32;              
                    let step_xy: f32  = leaf_size;             

                    // top del tronco
                    let top_trunk = y_trunk1 + trunk_size * 0.5_f32;

                    // alturas de cada nivel (cada uno apilado exactamente encima)
                    let y_lvl1 = top_trunk + leaf_size * 0.5_f32;          // base de hojas
                    let y_lvl2 = y_lvl1   + leaf_size;                      // medio
                
                    // Mascara 3x3 por nivel: 1=coloca cubo, 0=vacío
                    // Nivel 1 (base)
                    const L1: [[u8; 3]; 3] = [
                        [0, 1, 0],
                        [1, 1, 1],
                        [0, 1, 0],
                    ];
                    // Nivel 2
                    const L2: [[u8; 3]; 3] = [
                        [0, 0, 0],
                        [0, 1, 0],
                        [0, 0, 0],
                    ];

                    let mut place_layer = |y: f32, mask: [[u8;3];3]| {
                        for lr in 0..3 {
                            for lc in 0..3 {
                                if mask[lr][lc] == 1 {
                                    // columnas: -1,0,+1 ; filas: -1.5,-0.5,+0.5,+1.5
                                    let x_off = (lc as f32 - 1.0_f32) * step_xy;
                                    let z_off = (lr as f32 - 1.5_f32) * step_xy;
                                    add_cube(x + x_off, y, z + z_off, leaf_size, MAT_LEAVES);
                                }
                            }
                        }
                    };

                    place_layer(y_lvl1, L1);
                    place_layer(y_lvl2, L2);
                }
                'T' => {
                    // tronco sobre piso 1
                    add_cube(x, y_floor1, z, tile, MAT_GRASS_DARK);
                    add_cube(x, y_floor0, z, tile, MAT_GRASS); // base opcional

                    // tronco de 3 cubos
                    let trunk_size: f32 = 0.9_f32;
                    let y_trunk0 = y_floor1 + trunk_size * 0.5_f32;
                    let y_trunk1 = y_trunk0 + trunk_size;
                    let y_trunk2 = y_trunk1 + trunk_size;
                    add_cube(x, y_trunk0, z, trunk_size, MAT_WOOD);
                    add_cube(x, y_trunk1, z, trunk_size, MAT_WOOD);
                    add_cube(x, y_trunk2, z, trunk_size, MAT_WOOD);

                    // Copa: 3 niveles 
                    let leaf_size: f32 = 1.0_f32;              
                    let step_xy: f32  = leaf_size;             

                    // top del tronco
                    let top_trunk = y_trunk2 + trunk_size * 0.5_f32;

                    // alturas de cada nivel (cada uno apilado exactamente encima)
                    let y_lvl1 = top_trunk + leaf_size * 0.5_f32;          // base de hojas
                    let y_lvl2 = y_lvl1   + leaf_size;                      // medio
                    let y_lvl3 = y_lvl2   + leaf_size;                      // superior

                    // Mascara 3x4 por nivel: 1=coloca cubo, 0=vacío
                    // Nivel 1 (base)
                    const L1: [[u8; 3]; 4] = [
                        [1, 1, 1],
                        [1, 1, 1],
                        [1, 1, 1],
                        [0, 1, 0]
                    ];
                    // Nivel 2
                    const L2: [[u8; 3]; 4] = [
                        [0, 1, 0],
                        [1, 1, 1],
                        [0, 1, 0],
                        [0, 0, 0],
                    ];
                    // Nivel 3
                    const L3: [[u8; 3]; 4] = [
                        [0, 0, 0],
                        [0, 1, 0],
                        [0, 0, 0],
                        [0, 0, 0],
                    ];

                    let mut place_layer = |y: f32, mask: [[u8;3];4]| {
                        for lr in 0..4 {
                            for lc in 0..3 {
                                if mask[lr][lc] == 1 {
                                    // columnas: -1,0,+1 ; filas: -1.5,-0.5,+0.5,+1.5
                                    let x_off = (lc as f32 - 1.0_f32) * step_xy;
                                    let z_off = (lr as f32 - 1.5_f32) * step_xy;
                                    add_cube(x + x_off, y, z + z_off, leaf_size, MAT_LEAVES);
                                }
                            }
                        }
                    };

                    place_layer(y_lvl1, L1);
                    place_layer(y_lvl2, L2);
                    place_layer(y_lvl3, L3);
                }
                _ => {}
            }
        }
    }

    let mut camera = Camera::new(
        Vector3::new(1.5, 0.6, 12.0),
        Vector3::new(-0.5, 1.1, 1.0),
        Vector3::new(0.0, 6.0, 0.0),
    );
    camera.orbit(-0.3, 0.20);
    let rotation_speed = PI / 100.0;

    let light = Light::new(
        Vector3::new(0.0, 2.0, 4.0),
        Vector3::new(1.0, 1.0, 1.0),
        0.5,
    );

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        framebuffer.clear();

        // ... controles de cámara ...
        // velocidad angular en rad/seg (ajústala a tu gusto)
        let orbit_speed: f32 = 2.2; // ≈126°/s

        if window.is_key_down(KeyboardKey::KEY_LEFT)  { camera.orbit( orbit_speed * dt, 0.0); }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) { camera.orbit(-orbit_speed * dt, 0.0); }
        if window.is_key_down(KeyboardKey::KEY_UP)    { camera.orbit(0.0, -orbit_speed * dt); }
        if window.is_key_down(KeyboardKey::KEY_DOWN)  { camera.orbit(0.0,  orbit_speed * dt); }

        render(&mut framebuffer, &objects, &camera, &light, &texture_manager); 

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}