use rand::{prelude::Distribution, Rng};

use crate::{entity::EntityTemplate, vec::Vec2};

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

pub struct BoiTemplate<D: Distribution<f32>> {
    pub speed: D,
    pub vision: D,
    pub turning_speed: D,
}

impl<D: Distribution<f32>> EntityTemplate for BoiTemplate<D> {
    type Entity = Boi;

    fn spawn<R: Rng>(&self, rng: &mut R, position: &Vec2, facing: f32) -> Self::Entity {
        Boi {
            position: position.clone(),
            direction: facing,
            speed: self.speed.sample(rng),
            vision: self.vision.sample(rng),
            turning_speed: self.turning_speed.sample(rng),
        }
    }
}
