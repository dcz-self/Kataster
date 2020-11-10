/*! Last stander AI */
/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::asset::{ Assets, AssetServer };
use bevy::audio::Audio;
use bevy::ecs::{ Commands, Entity, Mut, Query, Res, ResMut, Without };
use bevy::math::{ Quat, Vec3 };
use bevy::sprite::ColorMaterial;
use bevy::transform::components::Transform;
use bevy_rapier2d::na::{ Point2, Vector2 };
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::RigidBodySet,
};
use super::brain;
use super::brain::{ Function, Neuron };
use super::components::{ weapon_trigger, AttachedToEntity, Borg, LooksAt, Mob, Weapon };
use super::geometry::{ angle_from, get_nearest };


use super::brain::Brain as _;


/// Process a fully connected layer
fn process_layer(neurons: &[Neuron], inputs: Vec<f32>) -> Vec<f32> {
    neurons.iter().map(|n| n.feed(&inputs)).collect()
}

/// Does nothing
/// Hardly even a neuron
fn dumb_neuron(synapse_count: u8) -> Neuron {
    Neuron {
        weights: (0..synapse_count).map(|_| 0.0).collect(),
        activation: Function::Linear,
    }
}


/// Brain used by the last stand hero
/// Uses a single hidden layer of neurons
#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    hidden_layer: Vec<Neuron>,
    output_layer: [Neuron; 1],
}

impl Brain {
    fn new_dumb(hidden_neurons: u8) -> Brain {
        Brain {
            hidden_layer: (0..hidden_neurons).map(|_| dumb_neuron(INPUT_COUNT + 1)).collect(),
            output_layer: [dumb_neuron(hidden_neurons)],
        }
    }
}

impl brain::Brain for Brain {
    type Inputs = Inputs;
    type Outputs = Outputs;
    fn process(&mut self, inputs: Inputs) -> Outputs {
        let inputs = vec![inputs.mob_rel_angle, 1.0];
        let hidden = process_layer(&self.hidden_layer, inputs);
        let outputs = process_layer(&self.output_layer, hidden);
        Outputs {
            walk: false,
            turn: 0.0,
            shoot: true,
            aim_rel_angle: outputs[0],
        }
    }
    fn mutate(self, strength: f32) -> Brain {
        // FIXME
        self
    }
}

pub struct Inputs {
    //mob_distance: f32,
    mob_rel_angle: f32,
}

const INPUT_COUNT: u8 = 1;

pub struct Outputs {
    walk: bool,
    /// Relative to walking direction
    turn: f32,
    shoot: bool,
    /// Relative to walking direction
    aim_rel_angle: f32,
}


pub fn think(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_output: Res<Audio>,
    mut bodies: ResMut<RigidBodySet>,
    mobs: Query<(&RigidBodyHandleComponent, &Mob)>,
    mut borgs: Query<(Entity, &RigidBodyHandleComponent, &Borg, Mut<Brain>)>,
    mut weapons: Query<(Without<LooksAt, Mut<Weapon>>, Mut<Transform>, &AttachedToEntity)>,
) {
    let mob_positions: Vec<_>
        = mobs.iter()
        .filter_map(|(body, _)| bodies.get(body.handle()))
        .map(|body| body.position.translation.vector.clone().into())
        .collect();

    for (entity, body, borg, mut brain) in borgs.iter_mut() {
        let mut body = bodies.get_mut(body.handle()).unwrap();
        let nearest = get_nearest(&body.position.translation.vector.into(), &mob_positions)
            .unwrap_or(Point2::new(0.0, 0.0));
        let rot = angle_from(&body.position, &nearest);
        let outputs = brain.process(Inputs {
            mob_rel_angle: rot,
        });
        // Apply outputs. Might be better to do this in a separate step.
        body.wake_up(true);
        body.angvel = outputs.turn;
        body.linvel = body.position.rotation.transform_vector(&Vector2::new(
            0.0,
            borg.speed * match outputs.walk {
                true => 1.0,
                false => 0.0,
            }
        ));
        let weapons = weapons.iter_mut().filter(|(_w, _t, parent)| parent.0 == entity);
        for (mut weapon, mut transform, _parent) in weapons {
            let abs_angle = body.position.rotation.angle() + outputs.aim_rel_angle;
            transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), abs_angle);
            if outputs.shoot {
                weapon_trigger(&mut weapon, &transform, &mut commands, &asset_server, &mut materials, &audio_output);
            }
        }
    }
}
