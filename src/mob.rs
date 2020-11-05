use bevy::prelude::{ Commands, Entity, Mut, Query, Res, ResMut, Time };
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::RigidBodySet,
};
use bevy_rapier2d::na::{ Point2, Rotation2, Vector2 };
use rand;
use rand::distributions::Uniform;
use std::collections::VecDeque;
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
#[derive(Debug, Clone)]
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

/// Same shape as Genotype, but weights reflect variances in values.
pub struct Variances {
    // body
    // brain
}

#[derive(Debug)]
pub struct GenePool {
    genotypes: VecDeque<Brain>,
}

impl GenePool {
    pub fn new_eden() -> GenePool {
        GenePool {
            genotypes: VecDeque::from(vec![
                Brain { weights: vec![f32::consts::TAU / 10.0, 1.0] }, // Adam
                Brain { weights: vec![0.0, 1.0] }, // Eve
            ]),
        }
    }

    pub fn spawn(&mut self) -> Brain {
        self.genotypes.pop_front()
            .unwrap_or_else(|| Brain::randomize())
    }

    pub fn preserve(&mut self, genotype: Brain) {
        self.genotypes.push_back(genotype)
    }

    // Calculate the variance of each element in the genotype
    pub fn get_variances(&self) -> Variances {
        panic!()
    }
}

pub fn think(
    time: Res<Time>,
    mut runstate: ResMut<RunState>,
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

        if turn_speed != body.angvel {
            body.wake_up(true);
            body.angvel = turn_speed;
            body.linvel = body.position.rotation.transform_vector(&Vector2::new(0.0, mob.speed));
        }
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
