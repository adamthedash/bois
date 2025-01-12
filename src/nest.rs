use crate::{entity::EntityTemplate, vec::Vec2};
use rand::{prelude::Distribution, Rng};

/// A Nest is some spawning point for an Entity.
pub struct Nest<R: Rng, D: Distribution<f32>, T: EntityTemplate> {
    pub rng: R,
    pub pos: D,
    pub direction: D,
    pub template: T,
}

impl<R: Rng, D: Distribution<f32>, T: EntityTemplate> Nest<R, D, T> {
    /// Spawn a new Entity near the nest
    pub fn spawn(&mut self) -> T::Entity {
        let position = Vec2 {
            x: self.pos.sample(&mut self.rng),
            y: self.pos.sample(&mut self.rng),
        };
        let direction = self.direction.sample(&mut self.rng);

        self.template.spawn(&mut self.rng, &position, direction)
    }
}
