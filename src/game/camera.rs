use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{Direction, VelocityController};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view: [[f32; 2]; 3],
}
impl From<&Camera> for CameraUniform {
    fn from(camera: &Camera) -> Self {
        let c = Self {
            view: [
                [2.0 / camera.view_size.width, 0.0],
                [0.0, 2.0 / camera.view_size.height],
                [
                    -2.0 * camera.position.x / camera.view_size.width,
                    -2.0 * camera.position.y / camera.view_size.height,
                ],
            ],
        };
        c
    }
}

#[derive(Clone)]
pub struct CameraDescriptor {
    pub position: Vector<f32>,
    pub view_size: PhysicalSize<f32>,
    pub speed: f32,
    pub acceleration_steps: u32,
}
impl From<&CameraDescriptor> for Camera {
    fn from(descriptor: &CameraDescriptor) -> Self {
        Self::new(
            descriptor.position.clone(),
            descriptor.view_size,
            descriptor.speed,
            descriptor.acceleration_steps,
        )
    }
}

pub struct Camera {
    position: Vector<f32>,
    velocity: Vector<f32>,
    max_speed: f32,
    decceleration_factor: f32,
    acceleration: VelocityController,
    view_size: PhysicalSize<f32>,
}
impl Camera {
    fn new(
        position: Vector<f32>,
        view_size: PhysicalSize<f32>,
        speed: f32,
        acceleration_steps: u32,
    ) -> Self {
        Self {
            position,
            velocity: Vector::new(0.0, 0.0, 0.0),
            max_speed: speed,
            decceleration_factor: 1.0 - 1.0 / acceleration_steps as f32,
            acceleration: VelocityController::new(speed / acceleration_steps as f32),
            view_size,
        }
    }

    pub fn update(&mut self) {
        let acceleration = self.acceleration.get_velocity();
        if acceleration == Vector::new(0.0, 0.0, 0.0) {
            self.velocity *= self.decceleration_factor;
        } else {
            self.velocity += acceleration;
            if self.velocity.magnitude_squared() >= self.max_speed {
                self.velocity = self.velocity.normalize() * self.max_speed;
            }
        }
        self.position += &self.velocity;
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(bytemuck::cast_slice(&CameraUniform::from(self).view));
        v
    }

    pub fn handle_key_input(&mut self, input: &KeyEvent) {
        if input.state == winit::event::ElementState::Released {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.acceleration.set_direction(Direction::Up, false);
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.acceleration.set_direction(Direction::Left, false);
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.acceleration.set_direction(Direction::Right, false);
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.acceleration.set_direction(Direction::Down, false);
                }
                _ => {}
            }
        } else if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.acceleration.set_direction(Direction::Up, true);
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.acceleration.set_direction(Direction::Left, true);
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.acceleration.set_direction(Direction::Right, true);
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.acceleration.set_direction(Direction::Down, true);
                }
                _ => {}
            }
        }
    }
}
