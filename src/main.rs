use game::MainState;
use ggez::{event, GameResult};

mod boi;
mod entity;
mod game;
mod nest;
mod strategy;
mod vec;

pub fn main() -> GameResult {
    let screen_scale = 3.; // How much bigger is the rendering than the world
    let padding = 100.; // 100 pixels padding on each side of the arena
    let arena_radius = 100.; // World units
    let fps = 5;

    let (ctx, event_loop) = ggez::ContextBuilder::new("bois", "adam")
        .window_setup(ggez::conf::WindowSetup::default().title("Bois"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(
            arena_radius * 2. * screen_scale + padding * 2.,
            arena_radius * 2. * screen_scale + padding * 2.,
        ))
        .build()?;
    let state = MainState::new(arena_radius, 100, screen_scale, fps, padding);
    event::run(ctx, event_loop, state)
}
