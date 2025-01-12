use std::{f32::consts::PI, ops::Sub};

use ggez::{
    event::{self, EventHandler},
    graphics::{self, Color, DrawMode, DrawParam, Mesh},
    Context, GameResult,
};
use rand::{distributions::Uniform, prelude::*};

#[derive(Debug)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn add_scalar(&self, other: f32) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }

    fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn div(&self, val: f32) -> Self {
        assert!(val > 0., "Divider must be positive! Got {}", val);
        Self {
            x: self.x / val,
            y: self.y / val,
        }
    }

    fn mul(&self, val: f32) -> Self {
        Self {
            x: self.x * val,
            y: self.y * val,
        }
    }

    /// Turn into unit vector
    fn normalise(&self) -> Self {
        let magnitude = (self.x.powi(2) + self.y.powi(2)).sqrt();
        if magnitude == 0. {
            Self { x: 0., y: 0. }
        } else {
            Self {
                x: self.x / magnitude,
                y: self.y / magnitude,
            }
        }
    }

    fn direction_radians(&self) -> f32 {
        self.y.atan2(self.x)
    }
}

#[derive(Debug)]
struct Boi {
    position: Vec2,
    direction: f32, // radians
    speed: f32,
    vision: f32,
    turning_speed: f32,
}

impl Boi {
    // Unit vector representing the direction the boi is facing
    fn direction_vector(&self) -> Vec2 {
        Vec2 {
            x: self.direction.cos(),
            y: self.direction.sin(),
        }
    }
}

/// Randomly generates Bois
struct Nest<R: Rng, D: Distribution<f32>> {
    rng: R,
    pos: D,
    direction: D,
    speed: D,
    vision: D,
    turning_speed: D,
}

impl<R: Rng, D: Distribution<f32>> Nest<R, D> {
    /// Spawns a new boi
    fn spawn(&mut self) -> Boi {
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
struct MainState {
    bois: Vec<Boi>,
    arena_centre: Vec2,
    arena_radius: f32,
    // draw stuff
    screen_scale: f32, // difference between world scale and draw scale
    padding: f32,      // padding around edge of world in pixels
    fps: u32,
    needs_render: bool,
}

impl MainState {
    fn new(arena_radius: f32, num_bois: usize, screen_scale: f32, fps: u32, padding: f32) -> Self {
        let arena_centre = Vec2 { x: 0., y: 0. };

        let mut nest = Nest {
            rng: thread_rng(),
            pos: Uniform::new(-arena_radius, arena_radius),
            direction: Uniform::new(0., 2. * PI),
            speed: Uniform::new(2., 3.),
            vision: Uniform::new(2., 10.),
            turning_speed: Uniform::new(0.1, 0.5),
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
                .enumerate()
                .map(|(i, boi1)| {
                    // See who's around
                    let nearbois = self
                        .bois
                        .iter()
                        .enumerate()
                        // Skip ourselves
                        .filter(|(j, _)| i != *j)
                        .map(|(_, boi)| boi)
                        // Within some distance
                        .filter(|boi2| boi1.position.distance(&boi2.position) <= boi1.vision)
                        .collect::<Vec<_>>();

                    // Calculate the ideal vector for each thing. Each of them should be a unit vector
                    // reprenting a direction to go in
                    let distances = nearbois
                        .iter()
                        .map(|boi2| boi1.position.distance(&boi2.position))
                        .collect::<Vec<_>>();

                    // Separation - Steer away from nearby bois - weight = 1 / distance
                    let separation = nearbois
                        .iter()
                        .map(|boi2| boi1.position.sub(&boi2.position).normalise())
                        .zip(&distances)
                        .map(|(boi2, distance)| boi2.div(*distance))
                        .reduce(|a, b| a.add(&b))
                        .map(|v| v.normalise());

                    // Alignment - Align direction with average of nearby bois - weight = constant
                    let alignment = nearbois
                        .iter()
                        .map(|boi2| boi2.direction_vector())
                        .reduce(|a, b| a.add(&b))
                        .map(|v| v.normalise());

                    // Cohesion - Steer towards the centre of gravity of nearbois
                    let cohesion = nearbois
                        .iter()
                        .map(|boi2| boi2.position.div(nearbois.len() as f32))
                        .reduce(|a, b| a.add(&b))
                        .map(|centre_of_gravity| centre_of_gravity.sub(&boi1.position).normalise());

                    // Don't escape the arena - Steer towards centre of arena if we're too far away.
                    // Weight = nothing until we're close to the edge, then ramps up exponentially
                    let distance_to_centre = self.arena_centre.distance(&boi1.position);
                    let escape = if distance_to_centre > self.arena_radius {
                        Some(self.arena_centre.sub(&boi1.position).normalise())
                    } else {
                        None
                    };

                    // Combine all the signals together
                    [
                        // Apply weighting for different factors, each of which may be null if there are no
                        // nearbois
                        separation.map(|x| x.mul(1.)),
                        alignment.map(|x| x.mul(1.)),
                        cohesion.map(|x| x.mul(1.)),
                        escape.map(|x| {
                            x.mul(distance_to_centre.sub(self.arena_radius).max(0.).powf(1.1))
                        }),
                    ]
                    .into_iter()
                    .flatten()
                    .reduce(|a, b| a.add(&b))
                    .unwrap_or_else(|| boi1.direction_vector())
                })
                .collect::<Vec<_>>();

            // Step 2) apply the decisions
            self.bois
                .iter_mut()
                .zip(decisions)
                .for_each(|(boi, new_direction)| {
                    let ideal_direction = new_direction.direction_radians();
                    // Figure out if we should turn left or Right
                    let mut delta = (ideal_direction - boi.direction).rem_euclid(2. * PI);
                    if delta > PI {
                        delta = PI - delta;
                    }

                    // Clip to max turning speed
                    let turn =
                        delta.signum() * delta.abs().min(boi.turning_speed / self.fps as f32);

                    boi.direction += turn;
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
