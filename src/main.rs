#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rayon::prelude::*;

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


// --- AÑADIDO: estado global de animación ---
struct SceneState {
    time: f32,
    scene_rotation: f32,
    camera_distance: f32, // distancia base desde center -> eye
    camera_center: Vector3,
}

impl SceneState {
    fn new() -> Self {
        SceneState {
            time: 0.0,
            scene_rotation: 0.0,
            camera_distance: 10.0,
            camera_center: Vector3::new(-0.5, 1.1, 1.0),
        }
    }

    // Actualiza posición del sol y colores según el tiempo
    fn update(&mut self, dt: f32) {
        self.time += dt;
        // Un ciclo completo cada 10 segundos
        let cycle = (self.time / 10.0) % 1.0;
        // Rotamos la escena
        self.scene_rotation += dt * (2.0 * PI / 30.0);
    }

    // Obtiene posición del sol (en esfera de radio 20)
    fn sun_position(&self) -> Vector3 {
        let cycle = (self.time / 10.0) % 1.0;
        let angle = cycle * 2.0 * PI - PI / 2.0; // sale por el este
        Vector3::new(
            20.0 * angle.cos(),
            20.0 * (angle.sin() + 1.0) * 0.5, // sube y baja
            0.0,
        )
    }

    fn background_color(&self) -> Color {
        let cycle = (self.time / 10.0) % 1.0; // 10 segundos por ciclo
        // 0.0 = noche, 0.25 = amanecer, 0.5 = día, 0.75 = atardecer, 1.0 = noche

        if cycle < 0.25 {
            // Noche → Amanecer (morado → rosa)
            let t = cycle / 0.25;
            // Morado oscuro → Rosa claro
            Color::new(
                (80.0 + 175.0 * t) as u8, // R: morado → rosa
                (40.0 + 140.0 * t) as u8, // G: oscuro → claro
                (100.0 + 155.0 * t) as u8, // B: morado → rosa
                255,
            )
        } else if cycle < 0.5 {
            // Amanecer → Día (rosa → celeste)
            let t = (cycle - 0.25) / 0.25;
            Color::new(
                (255.0 - 100.0 * t) as u8, // R: rosa → azul
                (180.0 + 75.0 * t) as u8, // G: más verde
                (255.0 - 50.0 * t) as u8, // B: rosa → celeste
                255,
            )
        } else if cycle < 0.75 {
            // Día → Atardecer
            let t = (cycle - 0.5) / 0.25;
            Color::new(
                (155.0 + 100.0 * (1.0 - t)) as u8, // Rosa de nuevo
                (255.0 - 75.0 * t) as u8,
                (205.0 - 105.0 * t) as u8,
                255,
            )
        } else {
            // Atardecer → Noche
            let t = (cycle - 0.75) / 0.25;
            Color::new(
                (255.0 - 175.0 * t) as u8, // Rosa → morado
                (180.0 - 140.0 * t) as u8,
                (100.0 + 55.0 * (1.0 - t)) as u8,
                255,
            )
        }
    }

    fn sun_color(&self) -> Color {
        let cycle = (self.time / 10.0) % 1.0;
        if cycle < 0.2 || cycle > 0.8 {
            Color::BLACK // noche → sol abajo
        } else if cycle < 0.3 {
            Color::new(255, 200, 100, 255) // amanecer
        } else if cycle < 0.7 {
            Color::new(255, 255, 200, 255) // día
        } else {
            Color::new(255, 200, 100, 255) // atardecer
        }
    }

    fn light_intensity(&self) -> f32 {
        let cycle = (self.time / 10.0) % 1.0;
        if cycle < 0.2 {
            0.1 * (cycle / 0.2) // amanecer
        } else if cycle < 0.5 {
            0.1 + 0.9 * ((cycle - 0.2) / 0.3) // sube a día
        } else if cycle < 0.8 {
            1.0 - 0.9 * ((cycle - 0.5) / 0.3) // baja a noche
        } else {
            0.1 * (1.0 - (cycle - 0.8) / 0.2) // atardecer
        }
    }
}

// --- NUEVA FUNCIÓN: rotar punto en XZ alrededor de Y ---
fn rotate_xz(point: Vector3, angle: f32) -> Vector3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Vector3::new(
        point.x * cos_a - point.z * sin_a,
        point.y,
        point.x * sin_a + point.z * cos_a,
    )
}
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
    texture_manager: &TextureManager,
    time: f32,
)-> Color {
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
    
    let mut diffuse_color = intersect.material.diffuse;
    if let Some(texture_id) = intersect.material.texture_id {
        if let Some(uv) = intersect.uv {
            if let Some(texture) = texture_manager.get_texture(texture_id) {
                let is_water = intersect.material.transparency > 0.1 && intersect.material.diffuse.y < 0.4;
                
                let (u, v) = if is_water {
                    let wave = (time * 2.0 + intersect.point.x * 0.5 + intersect.point.z * 0.5).sin() * 0.02;
                    let u = (uv.x + wave).fract();
                    let v = (uv.y + time * 0.1).fract();
                    (u, v)
                } else {
                    (uv.x, uv.y)
                };

                diffuse_color = color_to_vector3(texture.get_color(u, v));
            }
        }
    }

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


pub fn render<T: RayIntersect + Clone + Sync>(
    framebuffer: &mut Framebuffer, 
    objects: &[T], 
    camera: &Camera, 
    light: &Light,
    texture_manager: &TextureManager,
    time: f32,
    should_abort: &AtomicBool,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    let total_pixels = (framebuffer.width * framebuffer.height) as usize;
    let mut output_buffer = vec![Color::BLACK; total_pixels];

    let num_threads = rayon::current_num_threads();
    let chunk_size = (total_pixels + num_threads - 1) / num_threads;

    output_buffer
        .par_chunks_mut(chunk_size)
        .enumerate()
        .for_each(|(chunk_idx, chunk)| {
            if should_abort.load(Ordering::Relaxed) {
                return;
            }

            let start_idx = chunk_idx * chunk_size;
            for (i, pixel) in chunk.iter_mut().enumerate() {
                if should_abort.load(Ordering::Relaxed) {
                    return;
                }

                let flat_idx = start_idx + i;
                let x = (flat_idx % framebuffer.width as usize) as f32;
                let y = (flat_idx / framebuffer.width as usize) as f32;

                let screen_x = (2.0 * x) / width - 1.0;
                let screen_y = -(2.0 * y) / height + 1.0;
                let screen_x = screen_x * aspect_ratio * perspective_scale;
                let screen_y = screen_y * perspective_scale;

                let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
                let rotated_direction = camera.basis_change(&ray_direction);

                *pixel = cast_ray(
                    &camera.eye,
                    &rotated_direction,
                    objects,
                    light,
                    texture_manager,
                    time,
                );
            }
        });

    framebuffer.color_buffer = output_buffer;
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

    // ... antes del loop ...

let mut camera = Camera::new(
    Vector3::new(1.5, 0.6, 12.0),
    Vector3::new(-0.5, 1.1, 1.0),
    Vector3::new(0.0, 6.0, 0.0),
);
camera.orbit(-0.3, 0.20);

let light = Light::new(
    Vector3::new(0.0, 2.0, 4.0),
    Vector3::new(1.0, 1.0, 1.0),
    0.5,
);

let mut scene_state = SceneState::new();
camera.center = scene_state.camera_center;

let should_abort = Arc::new(AtomicBool::new(false)); // para threads

while !window.window_should_close() {
    let dt = window.get_frame_time();
    scene_state.update(dt);

    // --- ZOOM con rueda del mouse ---
    let wheel = window.get_mouse_wheel_move();
    if wheel != 0.0 {
        scene_state.camera_distance = (scene_state.camera_distance - wheel * 1.5).clamp(3.0, 30.0);
        let dir = (camera.eye - camera.center).normalized();
        camera.eye = camera.center + dir * scene_state.camera_distance;
    }

    // --- Control orbital ---
    let orbit_speed: f32 = 2.2;
    if window.is_key_down(KeyboardKey::KEY_LEFT)  { camera.orbit( orbit_speed * dt, 0.0); }
    if window.is_key_down(KeyboardKey::KEY_RIGHT) { camera.orbit(-orbit_speed * dt, 0.0); }
    if window.is_key_down(KeyboardKey::KEY_UP)    { camera.orbit(0.0, -orbit_speed * dt); }
    if window.is_key_down(KeyboardKey::KEY_DOWN)  { camera.orbit(0.0,  orbit_speed * dt); }

    // Actualizar luz y fondo
    let sun_pos = scene_state.sun_position();
    let light = Light::new(
        sun_pos,
        Vector3::new(1.0, 1.0, 1.0),
        scene_state.light_intensity(),
    );

    framebuffer.set_background_color(scene_state.background_color());
    framebuffer.clear();

    // Render con threads
    should_abort.store(false, Ordering::Relaxed);
    render(
        &mut framebuffer,
        &objects, // <-- sin rotación
        &camera,
        &light,
        &texture_manager,
        scene_state.time,
        &should_abort,
    );

    // Dibujar sol en pantalla
    {
        let mut drawing = window.begin_drawing(&raylib_thread);
        let sun_screen_x = (sun_pos.x / 20.0 * 0.5 + 0.5) * drawing.get_screen_width() as f32;
        let sun_screen_y = (1.0 - (sun_pos.y / 20.0 * 0.5 + 0.5)) * drawing.get_screen_height() as f32;

        if sun_pos.y > 0.0 {
            let radius = 20.0 * (sun_pos.y / 20.0).powf(2.0);
            drawing.draw_circle(
                sun_screen_x as i32,
                sun_screen_y as i32,
                radius.max(5.0) as f32,
                scene_state.sun_color(),
            );
        }
    }

    framebuffer.swap_buffers(&mut window, &raylib_thread);
}
}