use crate::{game::MainState, vec::Vec2};

/// Defines a strategy for a single entity.
pub trait Strategy {
    /// Decides what direction to turn towards
    fn decide(&self, game_state: &MainState) -> Vec2;

    /// Applies the decision (Eg. move). Also gives the time since last action in case there's a
    /// need to lerp some stuff.
    fn action(&mut self, time_step: f32, decision: &Vec2);
}
