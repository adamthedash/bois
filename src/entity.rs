use rand::Rng;

use crate::vec::Vec2;

/// Represents some template defining the constraints of an entity
pub trait EntityTemplate {
    type Entity;

    fn spawn<R: Rng>(&self, rng: &mut R, position: &Vec2, facing: f32) -> Self::Entity;
}
