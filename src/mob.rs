/*! Monster operations */
/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */


use bevy::prelude::{ Mut, Query, Res, ResMut, Time };
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::RigidBodySet,
};
use bevy_rapier2d::na::{ Point2, Rotation2, Vector2 };
use rand;
use rand::distributions::{ Bernoulli, Uniform };
use rand::distributions::weighted::WeightedIndex;
use rand_distr::StandardNormal;
use std::f32;
use super::arena;
use super::components::{ Borg, Mob };
use super::state::RunState;


use rand::distributions::Distribution;
use rand::Rng;


#[derive(Debug)]
pub struct Inputs {
    angle_to_player: f32,
    distance_to_borg: f32,
}

pub struct BrainCommands {
    turn_speed: f32,
}

/// Controls mobs by calculating a simple function, and being randomizeable.
#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    /// favorite_angle (bias), angle
    weights: Vec<f32>,
}

impl Brain {
    /// input = angle
    pub fn calculate(&self, inputs: Inputs) -> f32 {
        self.weights.iter()
            .zip(
                [
                    inputs.angle_to_player / f32::consts::PI,
                    inputs.distance_to_borg / arena::ARENA_HEIGHT as f32,
                    1.0, // bias
                ].iter()
            )
            .map(|(a, b)| a * b).sum()
    }
    
    fn randomize() -> Brain {
        let distribution = Uniform::new(-1.0, 1.0);
        Brain { weights: {
            (0..3).map(|_| distribution.sample(&mut rand::thread_rng()))
                .collect()
        }}
    }

    /// Alter values based on gene pool variance among the successful ones
    fn mutate(&self) -> Brain {
        let rng = &mut rand::thread_rng();
        Brain { weights: {
            self.weights.iter()
                .map(|v| v + rng.sample::<f32, _>(StandardNormal) * 0.05)
                .collect()
        }}
    }
}

pub type Genotype = Brain;

#[derive(Debug)]
pub struct GenePool {
    genotypes: Vec<(Genotype, f64)>,
    /// How often spawn a new blank (random) genotype.
    blank_frequency: f64,
}

impl GenePool {
    pub fn new_eden() -> GenePool {
        GenePool {
            genotypes: vec![
//                (Brain { weights: vec![f32::consts::TAU, 0.0, 1.0] }, 1.0), // Adam
                (Brain { weights: vec![10.0, 0.0, 0.0] }, 1.0),// Eve
            ],
            blank_frequency: 0.1,
        }
    }

    pub fn spawn(&mut self) -> Genotype {
        let blanks = Bernoulli::new(self.blank_frequency).unwrap();
        if blanks.sample(&mut rand::thread_rng()) || self.genotypes.is_empty() {
            Genotype::randomize()
        } else {
            let distribution = WeightedIndex::new(
                self.genotypes.iter().map(|(_k, v)| v)
            ).unwrap();
            self.genotypes
                .get_mut(distribution.sample(&mut rand::thread_rng()))
                .map(|(genotype, weight)| {
                    *weight /= 2.0;
                    genotype.clone()
                })
                .unwrap()
        }
    }

    pub fn preserve(&mut self, genotype: Genotype) {
        let index = self.genotypes.iter()
            .position(|(candidate, weight)| candidate == &genotype);
        match index {
            Some(idx) => { self.genotypes[idx].1 += 1.0 },
            None => self.genotypes.push((genotype, 1.0)),
        };
    }
}

pub fn think(
    mut bodies: ResMut<RigidBodySet>,
    mobs: Query<(&RigidBodyHandleComponent, &Mob)>,
    borgs: Query<(&RigidBodyHandleComponent, &Borg)>,
) {
    let borg_position = borgs.iter()
        .next() // Only take first borg. Should be expanded for multiplayer.
        .map(|(body, borg)| {
            let body = bodies.get(body.handle()).unwrap();
            Point2::from(body.position().translation.vector)
        })
        .unwrap_or(Point2::new(0.0, 0.0));
        
    for (body, mob) in mobs.iter() {
        let mut body = bodies.get_mut(body.handle()).unwrap();
        let point: Point2<f32> = body.position().inverse_transform_point(&borg_position);
        let inputs = Inputs {
            angle_to_player: {
                Rotation2::rotation_between(
                    &Vector2::new(0.0, 1.0),
                    &Vector2::new(point.x, point.y)
                ).angle()
            },
            distance_to_borg: {
                (
                    Point2::from(body.position().translation.vector)
                        - borg_position
                ).norm()
            },
        };
        //println!("{:?}", inputs);
        let turn_speed = mob.brain
            .calculate(inputs)
            .min(mob.rotation_speed)
            .max(-mob.rotation_speed);
        //println!("{}", turn_speed);
        body.set_angvel(turn_speed, true);
        body.set_linvel(body.position().rotation.transform_vector(&Vector2::new(0.0, mob.speed)), true);
    }
}


pub fn count_lifetime(
    runstate: Res<RunState>,
    time: Res<Time>,
    mut query: Query<Mut<Borg>>,
) {
    if !runstate.gamestate.current().is_live_arena() {
        return;
    }
    
    // FIXME: This is kind of inaccurate:
    // the delta when pausing will be different than unpausing.
    // Maybe switch to a constant tick.
    for mut borg in &mut query.iter_mut() {
        borg.time_alive += time.delta_seconds;
    }
}
