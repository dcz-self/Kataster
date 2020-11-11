/*! Monster operations */
/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */


use bevy::prelude::{ Commands, Entity, Mut, Query, Res, ResMut, Time };
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::RigidBodySet,
};
use bevy_rapier2d::na::{ Point2, Rotation2, Vector2 };
use rand;
use rand::distributions::{ Bernoulli, Uniform };
use rand::distributions::weighted::WeightedIndex;
use std::f32;
use super::components::{ Borg, Mob };
use super::state::{ GameState, RunState };


use rand::distributions::Distribution;


pub struct BrainInputs {
    angle_to_player: f32,
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
    pub fn calculate(&self, inputs: f32) -> f32 {
        self.weights[0] + self.weights[1] * inputs
    }
    
    fn randomize() -> Brain {
        let distribution = Uniform::new(-100.0, 100.0);
        Brain { weights: {
            (0..2).map(|_| distribution.sample(&mut rand::thread_rng()))
                .collect()
        }}
    }

    /// Alter values based on gene pool variance among the successful ones
    fn mutate(&self, variances: &Variances) -> Brain {
        self.clone()
    }
}

pub type Genotype = Brain;

/// Same shape as Genotype, but weights reflect variances in values.
pub struct Variances {
    // body
    // brain
}

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
                //(Brain { weights: vec![f32::consts::TAU / 10.0, 1.0] }, 1.0), // Adam
                //(Brain { weights: vec![0.0, 1.0] }, 1.0),// Eve
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

    // Calculate the variance of each element in the genotype
    pub fn get_variances(&self) -> Variances {
        panic!()
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
            Point2::from(body.position.translation.vector)
        })
        .unwrap_or(Point2::new(0.0, 0.0));
        
    for (body, mob) in mobs.iter() {
        let mut body = bodies.get_mut(body.handle()).unwrap();
        let point: Point2<f32> = body.position.inverse_transform_point(&borg_position);
        let rot = Rotation2::rotation_between(
            &Vector2::new(0.0, 1.0),
            &Vector2::new(point.x, point.y)
        );

        let turn_speed = mob.brain
            .calculate(rot.angle())
            .min(mob.rotation_speed)
            .max(-mob.rotation_speed);

        body.wake_up(true);
        body.angvel = turn_speed;
        body.linvel = body.position.rotation.transform_vector(&Vector2::new(0.0, mob.speed));
    }
}

pub fn expire(
    mut commands: Commands,
    runstate: Res<RunState>,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<Mob>)>,
) {
    if runstate.gamestate.is(GameState::Game) {
        for (entity, mut mob) in &mut query.iter_mut() {
            mob.lifeforce.tick(time.delta_seconds);
            if mob.lifeforce.finished {
                commands.despawn(entity);
            }
        }
    }
}

pub fn count_lifetime(
    runstate: Res<RunState>,
    time: Res<Time>,
    mut query: Query<Mut<Borg>>,
) {
    if !runstate.gamestate.is(GameState::Game) {
        return;
    }
    
    // FIXME: This is kind of inaccurate:
    // the delta when pausing will be different than unpausing.
    // Maybe switch to a constant tick.
    for mut borg in &mut query.iter_mut() {
        borg.time_alive += time.delta_seconds;
    }
}
