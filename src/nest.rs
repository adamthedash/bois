use crate::{boi::Boi, vec::Vec2};
use rand::{prelude::Distribution, Rng};

/// Randomly generates Bois
pub struct Nest<R: Rng, D: Distribution<f32>> {
    pub rng: R,
    pub pos: D,
    pub direction: D,
    pub speed: D,
    pub vision: D,
    pub turning_speed: D,
}

impl<R: Rng, D: Distribution<f32>> Nest<R, D> {
    /// Spawns a new boi
    pub fn spawn(&mut self) -> Boi {
        Boi {
            position: Vec2 {
                x: self.pos.sample(&mut self.rng),
                y: self.pos.sample(&mut self.rng),
            },
            direction: self.direction.sample(&mut self.rng),
            speed: self.speed.sample(&mut self.rng),
            vision: self.vision.sample(&mut self.rng),
            turning_speed: self.turning_speed.sample(&mut self.rng),
        }
    }
}
