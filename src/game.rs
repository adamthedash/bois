use crate::{
    boi::{Boi, BoiTemplate},
    nest::Nest,
    strategy::Strategy,
    vec::Vec2,
};
use std::f32::consts::PI;

use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawMode, DrawParam, Mesh},
    Context, GameResult,
};
use rand::{distributions::Uniform, prelude::*};

pub struct MainState {
    // Game state stuff
    pub bois: Vec<Boi>,
    pub arena_centre: Vec2,
    pub arena_radius: f32,

    // Rendering stuff
    screen_scale: f32, // difference between world scale and draw scale
    padding: f32,      // padding around edge of world in pixels
    fps: u32,
    needs_render: bool,
}

impl MainState {
    pub fn new(
        arena_radius: f32,
        num_bois: usize,
        screen_scale: f32,
        fps: u32,
        padding: f32,
    ) -> Self {
        let arena_centre = Vec2 { x: 0., y: 0. };

        let mut nest = Nest {
            rng: thread_rng(),
            pos: Uniform::new(-arena_radius, arena_radius),
            direction: Uniform::new(0., 2. * PI),
            template: BoiTemplate {
                speed: Uniform::new(2., 3.),
                vision: Uniform::new(2., 10.),
                turning_speed: Uniform::new(0.1, 0.5),
            },
        };

        let bois = (0..num_bois).map(|_| nest.spawn()).collect::<Vec<_>>();

        Self {
            bois,
            arena_radius,
            arena_centre,
            screen_scale,
            fps,
            padding,
            needs_render: true,
        }
    }
    /// Converts a position in world space to canvas space
    fn world_to_canvas(&self, vec: &Vec2) -> Vec2 {
        vec.add_scalar(self.arena_radius)
            .mul(self.screen_scale)
            .add_scalar(self.padding)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(10) {
            println!("--- Upate ---");
            // Step 1) decision time
            let decisions = self
                .bois
                .iter()
                .map(|boi| boi.decide(self))
                .collect::<Vec<_>>();

            // Step 2) apply the decisions
            self.bois
                .iter_mut()
                .zip(decisions)
                .for_each(|(boi, new_direction)| {
                    boi.action(1. / self.fps as f32, &new_direction);
                });

            // Step 3) Advance time
            self.bois.iter_mut().for_each(|boi| {
                boi.position = boi
                    .position
                    .add(&boi.direction_vector().mul(boi.speed / self.fps as f32))
            });

            // Step 4) Apply consequences (Eg. bois being gobbled)

            self.needs_render = true;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if self.needs_render {
            println!("--- Draw ---");
            let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(128, 218, 235));
            // Meshes are drawn at a higher resolution first so they
            // don't look blocky
            let mesh_raster_scale = 100.;

            // Debug markers
            let world_circle = Mesh::new_circle(
                ctx,
                DrawMode::stroke(1. * mesh_raster_scale),
                [0., 0.],
                self.arena_radius * mesh_raster_scale,
                2.,
                Color::BLACK,
            )?;
            let arena_pos = self.world_to_canvas(&self.arena_centre);
            canvas.draw(
                &world_circle,
                DrawParam::default()
                    .dest([arena_pos.x, arena_pos.y])
                    .scale([
                        self.screen_scale / mesh_raster_scale,
                        self.screen_scale / mesh_raster_scale,
                    ]),
            );

            // Create a basic shape that will be copied/transformed to each boi
            let body = Mesh::new_polygon(
                ctx,
                DrawMode::fill(),
                &[
                    [1. * mesh_raster_scale, 0. * mesh_raster_scale], // Tip of the triangle (points forward)
                    [-1. * mesh_raster_scale, 3. / 5. * mesh_raster_scale], // Left base of the triangle
                    [-1. * mesh_raster_scale, -3. / 5. * mesh_raster_scale], // Right base of the triangle
                ],
                Color::RED,
            )?;

            let vision_circle = Mesh::new_circle(
                ctx,
                DrawMode::stroke(0.1 * mesh_raster_scale),
                [0., 0.],
                1. * mesh_raster_scale,
                2.,
                Color::new(0., 0., 0., 0.2),
            )?;

            //let circle = Mesh::new_circle(ctx, DrawMode::fill(), [0., 0.], 2., 2., Color::RED)?;
            self.bois.iter().for_each(|boi| {
                // Draw boi
                let position = self.world_to_canvas(&boi.position);
                canvas.draw(
                    &body,
                    DrawParam::default()
                        .dest([position.x, position.y])
                        .rotation(boi.direction)
                        .scale([
                            self.screen_scale / mesh_raster_scale,
                            self.screen_scale / mesh_raster_scale,
                        ]),
                );

                // Draw vision
                canvas.draw(
                    &vision_circle,
                    DrawParam::default().dest([position.x, position.y]).scale([
                        self.screen_scale * boi.vision / mesh_raster_scale,
                        self.screen_scale * boi.vision / mesh_raster_scale,
                    ]),
                );
            });

            canvas.finish(ctx)?;

            self.needs_render = false;
        }
        Ok(())
    }
}
