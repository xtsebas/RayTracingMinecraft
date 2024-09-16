use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    has_changed: bool,
    pub pos: Vec3,   // Posición de la cámara en el espacio 3D
    pub dir: Vec3,   // Dirección de la cámara (hacia adelante)
    pub fov: f32,  
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3, pos: Vec3, dir: Vec3, fov: f32) -> Self {
        Camera {
            eye,
            center,
            up,
            has_changed: true,
            pos,
            dir,
            fov
       }
    }

    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward).normalize();

        let rotated = 
            vector.x * right +
            vector.y * up +
            -vector.z * forward;

        rotated.normalize()
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        let current_yaw = radius_vector.z.atan2(radius_vector.x);

        let radius_xz = (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let new_pitch = (current_pitch + delta_pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        let new_eye = self.center + Vec3::new(
            radius * new_yaw.cos() * new_pitch.cos(),
            -radius * new_pitch.sin(),
            radius * new_yaw.sin() * new_pitch.cos()
        );

        self.eye = new_eye;
        self.has_changed = true;
    }

    pub fn is_changed(&mut self) -> bool {
        if self.has_changed {
            self.has_changed = false;
            return true;
        }
        false
    }

    pub fn zoom(&mut self, amount: f32) {
        let direction = (self.center - self.eye).normalize(); // Direccion de la cámara
        self.eye += direction * amount; // Ajustar la posición de la cámara en base a la dirección
        self.has_changed = true; // Marcar que ha habido un cambio
    }
}