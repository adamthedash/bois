use crate::{game::MainState, vec::Vec2};

/// Defines a strategy for a single entity.
pub trait Strategy {
    /// Decides what direction to turn towards
    fn decide(&self, game_state: &MainState) -> Vec2;
}
