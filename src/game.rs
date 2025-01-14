use crate::{
    boi::{Boi, BoiTemplate, Species},
    nest::Nest,
    render::{Assets, RenderState},
    strategy::Strategy,
    vec::Vec2,
};
use std::f32::consts::PI;

use geo_index::kdtree::{KDTree, KDTreeBuilder, KDTreeIndex};
use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawParam, Drawable},
    Context, GameResult,
};
use rand::{distributions::Uniform, prelude::*};

pub struct MainState {
    // Game state stuff
    pub bois: Vec<Boi>,
    pub boi_tree: KDTree<f32>,
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

        // Build a K-D tree of the bois
        let mut tree_builder = KDTreeBuilder::new(bois.len() as u32);
        bois.iter().for_each(|boi| {
            tree_builder.add(boi.position.x, boi.position.y);
        });
        let boi_tree = tree_builder.finish();

        Ok(Self {
            bois,
            arena_radius,
            arena_centre,
            render,
            boi_tree,
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
        while ctx.time.check_update_time(self.render.fps) {
            println!("--- Upate ---");

            //Build a K-D tree of the bois
            self.boi_tree = self
                .bois
                .iter()
                .fold(
                    KDTreeBuilder::new(self.bois.len() as u32),
                    |mut tree, boi| {
                        tree.add(boi.position.x, boi.position.y);
                        tree
                    },
                )
                .finish();

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
            //Build a K-D tree of the bois
            self.boi_tree = self
                .bois
                .iter()
                .fold(
                    KDTreeBuilder::new(self.bois.len() as u32),
                    |mut tree, boi| {
                        tree.add(boi.position.x, boi.position.y);
                        tree
                    },
                )
                .finish();

            // Kill off any bois that got caught
            let keep_bois = self
                .bois
                .iter()
                .map(|boi| {
                    // Predators always stay alive
                    if boi.species == Species::Predator {
                        return true;
                    }

                    // For Prey, we check if there's any nearby predators
                    let nearby_predator = self
                        .boi_tree
                        // Query the tree since it's quicker - kill range is 1 unit
                        .within(boi.position.x, boi.position.y, 1.)
                        .into_iter()
                        // Get the bois based on the spatial query
                        .map(|i| self.bois.get(i as usize).expect("Got invalid boi index!"))
                        // Skip ourselves. todo: This is comparing that the entities are the same in memory,
                        // this might bite me in the ass later. Probably better to do some unique entity IDs on
                        // spawn instead.
                        .filter(|boi2| !std::ptr::eq(*boi2, boi))
                        // Check if there's any predators
                        .any(|boi2| boi2.species == Species::Predator);

                    !nearby_predator
                })
                .collect::<Vec<_>>();

            // Kill em off
            self.bois = std::mem::take(&mut self.bois)
                .into_iter()
                .zip(keep_bois)
                .filter_map(|(boi, keep)| if keep { Some(boi) } else { None })
                .collect();

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
                        ])
                        // Change the colour depending on the species
                        .color(match boi.species {
                            Species::Predator => Color::RED,
                            Species::Prey => Color::GREEN,
                        }),
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
