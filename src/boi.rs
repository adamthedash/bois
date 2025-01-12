use crate::vec::Vec2;

#[derive(Debug)]
pub struct Boi {
    pub position: Vec2,
    pub direction: f32, // radians
    pub speed: f32,
    pub vision: f32,
    pub turning_speed: f32,
}

impl Boi {
    // Unit vector representing the direction the boi is facing
    pub fn direction_vector(&self) -> Vec2 {
        Vec2 {
            x: self.direction.cos(),
            y: self.direction.sin(),
        }
    }
}
