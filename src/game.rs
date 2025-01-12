use crate::{
    boi::{Boi, BoiTemplate},
    nest::Nest,
    render::{Assets, RenderState},
    strategy::Strategy,
    vec::Vec2,
};
use std::f32::consts::PI;

use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawParam, Drawable},
    Context, GameResult,
};
use rand::{distributions::Uniform, prelude::*};

pub struct MainState {
    // Game state stuff
    pub bois: Vec<Boi>,
    pub arena_centre: Vec2,
    pub arena_radius: f32,

    // Rendering stuff
    render: RenderState,
}

impl MainState {
    pub fn new(
        ctx: &mut Context,
        arena_radius: f32,
        num_bois: usize,
        screen_scale: f32,
        fps: u32,
        padding: f32,
    ) -> GameResult<Self> {
        // Load all the assets once at the start
        let mesh_raster_scale = 100.;
        let assets = Assets::load(ctx, mesh_raster_scale, arena_radius)?;
        let render = RenderState {
            assets,
            screen_scale,
            padding,
            fps,
            needs_render: true,
            mesh_raster_scale,
        };

        // Spawn a bunch of Bois
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

        Ok(Self {
            bois,
            arena_radius,
            arena_centre,
            render,
        })
    }
    /// Converts a position in world space to canvas space
    fn world_to_canvas(&self, vec: &Vec2) -> Vec2 {
        vec.add_scalar(self.arena_radius)
            .mul(self.render.screen_scale)
            .add_scalar(self.render.padding)
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
                    boi.action(1. / self.render.fps as f32, &new_direction);
                });

            // Step 3) Advance time
            self.bois.iter_mut().for_each(|boi| {
                boi.position = boi.position.add(
                    &boi.direction_vector()
                        .mul(boi.speed / self.render.fps as f32),
                )
            });

            // Step 4) Apply consequences (Eg. bois being gobbled)

            self.render.needs_render = true;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if self.render.needs_render {
            println!("--- Draw ---");
            let mut canvas = graphics::Canvas::from_frame(ctx, Color::from_rgb(128, 218, 235));

            // Debug - Arena boundaries
            let arena_pos = self.world_to_canvas(&self.arena_centre);
            canvas.draw(
                &self.render.assets.arena,
                DrawParam::default()
                    .dest([arena_pos.x, arena_pos.y])
                    .scale([self.render.base_scale(), self.render.base_scale()]),
            );

            self.bois.iter().for_each(|boi| {
                // Draw boi
                let position = self.world_to_canvas(&boi.position);
                let bbox = self.render.assets.boi.dimensions(ctx).unwrap().size();
                canvas.draw(
                    &self.render.assets.boi,
                    DrawParam::default()
                        .dest([position.x, position.y])
                        // +PI/2 since our image is 90 degrees rotated left
                        .rotation(boi.direction + PI / 2.)
                        // Align image centre with Boi centre
                        .offset([0.5, 0.5])
                        // Handle scaling specifically for this image (see asset loading section)
                        .scale([
                            10. * self.render.screen_scale / bbox.x,
                            10. * self.render.screen_scale / bbox.y,
                        ]),
                );

                // Debug - boi vision
                canvas.draw(
                    &self.render.assets.vision,
                    DrawParam::default().dest([position.x, position.y]).scale([
                        self.render.base_scale() * boi.vision,
                        self.render.base_scale() * boi.vision,
                    ]),
                );
            });

            canvas.finish(ctx)?;

            self.render.needs_render = false;
        }
        Ok(())
    }
}
