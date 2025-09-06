use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Vec<Color>,
    pub background_color: Color,
    current_color: Color,
    pub light_buffer: Vec<f32>
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let buffer_size = (width * height) as usize;
        let color_buffer = vec![background_color; buffer_size];
        
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            light_buffer: vec![0.0; buffer_size]
        }
    }

    pub fn clear(&mut self) {
        // Optimización: usar fill es mucho más rápido
        self.color_buffer.fill(self.background_color);
    }
    
    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.color_buffer.len() {
                self.color_buffer[index] = self.current_color;
            }
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }
    
    pub fn swap_buffers(&mut self, window: &mut RaylibHandle, raylib_thread: &RaylibThread) {
        let screen_width = window.get_screen_width() as f32;
        let screen_height = window.get_screen_height() as f32;
        
        // Calcular escala una sola vez
        let scale_x = screen_width / self.width as f32;
        let scale_y = screen_height / self.height as f32;
        let scale = f32::min(scale_x, scale_y);
        
        let scaled_width = self.width as f32 * scale;
        let scaled_height = self.height as f32 * scale;
        let offset_x = (screen_width - scaled_width) / 2.0;
        let offset_y = (screen_height - scaled_height) / 2.0;

        let mut rendering = window.begin_drawing(raylib_thread);
        rendering.clear_background(self.background_color);

        // Dibujar solo píxeles que han cambiado
        // o usar rectángulos en lugar de píxeles individuales
        
        if scale >= 2.0 {
            // Si la escala es >= 2, usar rectángulos
            let pixel_size = scale as i32;
            
            for y in 0..self.height {
                for x in 0..self.width {
                    let index = (y * self.width + x) as usize;
                    if index < self.color_buffer.len() {
                        let color = self.color_buffer[index];
                        if color != self.background_color { // Solo dibujar píxeles no vacíos
                            let screen_x = offset_x + (x as f32 * scale);
                            let screen_y = offset_y + (y as f32 * scale);
                            
                            rendering.draw_rectangle(
                                screen_x as i32,
                                screen_y as i32,
                                pixel_size,
                                pixel_size,
                                color
                            );
                        }
                    }
                }
            }
        } else {
            // Si la escala es pequeña, usar píxeles
            for y in 0..self.height {
                for x in 0..self.width {
                    let index = (y * self.width + x) as usize;
                    if index < self.color_buffer.len() {
                        let color = self.color_buffer[index];
                        if color != self.background_color {
                            let screen_x = offset_x + (x as f32 * scale);
                            let screen_y = offset_y + (y as f32 * scale);
                            
                            rendering.draw_pixel(screen_x as i32, screen_y as i32, color);
                        }
                    }
                }
            }
        }
    }
}
