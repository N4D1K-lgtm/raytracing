use crate::camera::Camera;
use crate::vec3::Vec3;

pub struct CameraController {
    // Input state
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    // Camera parameters
    pub position: Vec3,
    pub yaw: f64,   // Rotation around Y axis (radians)
    pub pitch: f64, // Rotation around X axis (radians)

    // Settings
    pub movement_speed: f64,    // Units per second
    pub mouse_sensitivity: f64, // Radians per pixel
}

impl CameraController {
    pub fn new(position: Vec3, yaw: f64, pitch: f64) -> Self {
        CameraController {
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            position,
            yaw,
            pitch,
            movement_speed: 2.0,
            mouse_sensitivity: 0.002,
        }
    }

    pub fn process_keyboard(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        use winit::keyboard::KeyCode;
        match key {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyA => self.left = pressed,
            KeyCode::KeyD => self.right = pressed,
            KeyCode::Space => self.up = pressed,
            KeyCode::ShiftLeft => self.down = pressed,
            _ => {}
        }
    }

    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw += delta_x * self.mouse_sensitivity;
        self.pitch -= delta_y * self.mouse_sensitivity;

        // Clamp pitch to prevent camera flip
        let max_pitch = std::f64::consts::FRAC_PI_2 - 0.01;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f64) -> bool {
        let mut moved = false;

        // Calculate forward, right, and up vectors from yaw and pitch
        let forward = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
        .normalized();

        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()).normalized();
        let up = Vec3::new(0.0, 1.0, 0.0);

        // Calculate velocity based on input
        let mut velocity = Vec3::ZERO;
        if self.forward {
            velocity += forward;
            moved = true;
        }
        if self.backward {
            velocity -= forward;
            moved = true;
        }
        if self.right {
            velocity += right;
            moved = true;
        }
        if self.left {
            velocity -= right;
            moved = true;
        }
        if self.up {
            velocity += up;
            moved = true;
        }
        if self.down {
            velocity -= up;
            moved = true;
        }

        // Normalize velocity if moving diagonally
        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalized() * self.movement_speed * delta_time;
            self.position += velocity;
        }

        // Update camera orientation
        camera.update_orientation(self.position, forward, right, up);

        moved
    }
}
