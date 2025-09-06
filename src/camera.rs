use raylib::prelude::*;

pub struct Camera {
    pub eye: Vector3,     // Camera position in world coordinates
    pub center: Vector3,  // Point the camera is looking at
    pub up: Vector3,      // Up direction (initially world up, gets orthonormalized)
    pub forward: Vector3, // Direction camera is facing (computed from eye->center)
    pub right: Vector3,   // Right direction (perpendicular to forward and up)
}

impl Camera {
    pub fn new(eye: Vector3, center: Vector3, up: Vector3) -> Self {
        let mut camera = Camera {
            eye,
            center,
            up,
            forward: Vector3::zero(), 
            right: Vector3::zero(),   
        };
       
        camera.update_basis_vectors();
        camera
    }

    /// Recomputes the camera
    pub fn update_basis_vectors(&mut self) {
        self.forward = (self.center - self.eye).normalized();
        self.right = self.forward.cross(self.up).normalized();
        self.up = self.right.cross(self.forward);
    }

    /// Rotates the camera around the center point (orbital camera movement)
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let relative_pos = self.eye - self.center;
        let radius = relative_pos.length();
        let current_yaw = relative_pos.z.atan2(relative_pos.x);
        let current_pitch = (relative_pos.y / radius).asin();

        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5); // Clamp to avoid gimbal lock
   
        let cos_pitch = new_pitch.cos();
        let new_relative_pos = Vector3::new(
            radius * cos_pitch * new_yaw.cos(),  // X component
            radius * new_pitch.sin(),            // Y component  
            radius * cos_pitch * new_yaw.sin(),  // Z component
        );
        
        self.eye = self.center + new_relative_pos;
        
        self.update_basis_vectors();
    }

    /// Transforms a vector from camera space to world space using basis vectors
    pub fn basis_change(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            v.x * self.right.x + v.y * self.up.x - v.z * self.forward.x,
            v.x * self.right.y + v.y * self.up.y - v.z * self.forward.y,
            v.x * self.right.z + v.y * self.up.z - v.z * self.forward.z,
        )
    }
}