use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{Direction, VelocityController};

use super::entity::EntityName;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view: [[f32; 2]; 3],
}
impl From<&Camera> for CameraUniform {
    fn from(camera: &Camera) -> Self {
        let x = camera.position.x + camera.offset_position.x;
        let y = camera.position.y + camera.offset_position.y;
        let c = Self {
            view: [
                [2.0 / camera.view_size.width, 0.0],
                [0.0, 2.0 / camera.view_size.height],
                [
                    -2.0 * x / camera.view_size.width,
                    -2.0 * y / camera.view_size.height,
                ],
            ],
        };
        c
    }
}

#[derive(Clone)]
pub struct CameraDescriptor {
    pub view_size: PhysicalSize<f32>,
    pub speed: f32,
    pub acceleration_steps: u32,
    pub target_entity: EntityName,
    pub max_offset_position: f32,
}
impl From<&CameraDescriptor> for Camera {
    fn from(descriptor: &CameraDescriptor) -> Self {
        Self::new(descriptor)
    }
}

pub struct Camera {
    position: Vector<f32>,
    offset_position: Vector<f32>,
    max_offset: f32,
    velocity: Vector<f32>,
    max_speed: f32,
    decceleration_factor: f32,
    acceleration: VelocityController,
    view_size: PhysicalSize<f32>,
    pub target_entity: EntityName,
}
impl Camera {
    fn new(descriptor: &CameraDescriptor) -> Self {
        Self {
            position: Vector::new(0.0, 0.0, 0.0),
            offset_position: Vector::new(0.0, 0.0, 0.0),
            max_offset: descriptor.max_offset_position.powi(2),
            velocity: Vector::new(0.0, 0.0, 0.0),
            max_speed: descriptor.speed,
            decceleration_factor: 1.0 - 1.0 / descriptor.acceleration_steps as f32,
            acceleration: VelocityController::new(
                descriptor.speed / descriptor.acceleration_steps as f32,
            ),
            view_size: descriptor.view_size,
            target_entity: descriptor.target_entity.clone(),
        }
    }

    pub fn update(&mut self, target_position: &Vector<f32>) {
        // let acceleration = self.acceleration.get_velocity();
        // if acceleration == Vector::new(0.0, 0.0, 0.0) {
        //     self.velocity*= self.decceleration_factor;
        // } else {
        //     self.velocity += acceleration;
        //     if self.velocity.magnitude_squared() >= self.max_speed {
        //         self.velocity = self.velocity.normalize() * self.max_speed;
        //     }
        //     self.offset_position = self.velocity.clone();
        // }
        self.position = target_position.clone();
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
