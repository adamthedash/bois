use std::{f32::consts::PI, ops::Sub};

use geo_index::kdtree::KDTreeIndex;
use rand::{prelude::Distribution, seq::SliceRandom, Rng};

use crate::{entity::EntityTemplate, strategy::Strategy, vec::Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Species {
    Predator,
    Prey,
}

#[derive(Debug)]
pub struct Boi {
    pub species: Species,
    pub position: Vec2,
    pub direction: f32, // radians
    pub speed: f32,
    pub vision: f32,
    pub turning_speed: f32,
}

impl Boi {
    // Unit vector representing the direction the boi is facing
    pub fn direction_vector(&self) -> Vec2 {
        Vec2 {
            x: self.direction.cos(),
            y: self.direction.sin(),
        }
    }
}

pub struct BoiTemplate<D: Distribution<f32>> {
    pub speed: D,
    pub vision: D,
    pub turning_speed: D,
}

impl<D: Distribution<f32>> EntityTemplate for BoiTemplate<D> {
    type Entity = Boi;

    fn spawn<R: Rng>(&self, rng: &mut R, position: &Vec2, facing: f32) -> Self::Entity {
        let choices = [(1., Species::Predator), (5., Species::Prey)];
        let choice = choices.choose_weighted(rng, |item| item.0).unwrap().1;

        Boi {
            position: *position,
            direction: facing,
            speed: self.speed.sample(rng),
            vision: self.vision.sample(rng),
            turning_speed: self.turning_speed.sample(rng),
            species: choice,
        }
    }
}

impl Strategy for Boi {
    fn decide(&self, game_state: &crate::game::MainState) -> Vec2 {
        // See who's around
        let nearbois = game_state
            // Query the tree since it's quicker
            .boi_tree
            .within(self.position.x, self.position.y, self.vision)
            .into_iter()
            // Get the bois based on the spatial query
            .map(|i| {
                game_state
                    .bois
                    .get(i as usize)
                    .expect("Got invalid boi index!")
            })
            // Skip ourselves. todo: This is comparing that the entities are the same in memory,
            // this might bite me in the ass later. Probably better to do some unique entity IDs on
            // spawn instead.
            .filter(|boi| !std::ptr::eq(*boi, self))
            // Limit to 100 nearbois if we have way too many
            //.take(10)
            // Within some distance
            .collect::<Vec<_>>();

        // Split bois into friends & foes
        let (friends, enemies) =
            nearbois
                .into_iter()
                .fold((vec![], vec![]), |(mut friends, mut enemies), boi| {
                    if boi.species == self.species {
                        friends.push(boi);
                    } else {
                        enemies.push(boi);
                    }

                    (friends, enemies)
                });

        // Calculate the ideal vector for each thing. Each of them should be a unit vector
        // reprenting a direction to go in

        // Cache distances - todo: use result of query instead if crate maintainer implements it!
        let friend_distances = friends
            .iter()
            .map(|boi| self.position.distance(&boi.position))
            .collect::<Vec<_>>();
        let enemy_distances = enemies
            .iter()
            .map(|boi| self.position.distance(&boi.position))
            .collect::<Vec<_>>();

        // Separation - Steer away from nearby bois - weight = 1 / distance
        let separation = friends
            .iter()
            .map(|boi| self.position.sub(&boi.position).normalise())
            .zip(&friend_distances)
            .map(|(boi, distance)| boi.div(*distance))
            .reduce(|a, b| a.add(&b))
            .map(|v| v.normalise());

        // Alignment - Align direction with average of nearby bois - weight = constant
        let alignment = friends
            .iter()
            .map(|boi| boi.direction_vector())
            .reduce(|a, b| a.add(&b))
            .map(|v| v.normalise());

        // Cohesion - Steer towards the centre of gravity of nearbois
        let cohesion = friends
            .iter()
            .map(|boi| boi.position.div(friends.len() as f32))
            .reduce(|a, b| a.add(&b))
            .map(|centre_of_gravity| centre_of_gravity.sub(&self.position).normalise());

        // Attack - Steer towards the nearest prey boi
        let attack = enemy_distances
            .iter()
            .zip(&enemies)
            .filter(|(_, boi)| boi.species == Species::Prey)
            .min_by(|(distance1, _), (distance2, _)| distance1.total_cmp(distance2))
            .map(|(_, boi)| boi.position.sub(&self.position).normalise());

        // Defend - Steer away from the nearest predator boi
        let defend = enemy_distances
            .iter()
            .zip(&enemies)
            .filter(|(_, boi)| boi.species == Species::Predator)
            .min_by(|(distance1, _), (distance2, _)| distance1.total_cmp(distance2))
            .map(|(_, boi)| self.position.sub(&boi.position).normalise());

        // Don't escape the arena - Steer towards centre of arena if we're too far away.
        // Weight = nothing until we're close to the edge, then ramps up exponentially
        let distance_to_centre = game_state.arena_centre.distance(&self.position);
        let escape = if distance_to_centre > game_state.arena_radius {
            Some(game_state.arena_centre.sub(&self.position).normalise())
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
                x.mul(
                    distance_to_centre
                        .sub(game_state.arena_radius)
                        .max(0.)
                        .powf(1.1),
                )
            }),
            attack.map(|x| {
                x.mul(if self.species == Species::Predator {
                    5. // Predators always chase pray as top priority
                } else {
                    0.
                })
            }),
            defend.map(|x| {
                x.mul(if self.species == Species::Prey {
                    5. // Prey always run away from predators as top priority
                } else {
                    0.
                })
            }),
        ]
        .into_iter()
        // Discard null signals
        .flatten()
        // Weighted avg
        .reduce(|a, b| a.add(&b))
        // If there's no signal, keep on truckin'
        .unwrap_or_else(|| self.direction_vector())
    }

    fn action(&mut self, time_step: f32, direction: &Vec2) {
        // Figure out if we should turn left or Right
        let mut delta = (direction.direction_radians() - self.direction).rem_euclid(2. * PI);
        if delta > PI {
            delta = PI - delta;
        }

        // Clip to max turning speed
        delta = delta.signum() * delta.abs().min(self.turning_speed * time_step);

        self.direction += delta;
    }
}
