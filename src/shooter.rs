/*! Last stander AI */
/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::asset::AssetServer;
use bevy::audio::Audio;
use bevy::ecs::{ Commands, Entity, Mut, Query, Res, ResMut, Without };
use bevy::math::{ Quat, Vec3 };
use bevy::transform::components::Transform;
use bevy_rapier2d::na::{ Point2, Vector2 };
use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::RigidBodySet,
};
use rand::distributions::Bernoulli;
use rand_distr::StandardNormal;
use std::f32;
use std::fmt;
use std::io;
use super::assets;
use super::brain;
use super::brain::{ Function, Neuron };
use super::components::{ weapon_trigger, AttachedToEntity, Borg, LooksAt, Mob, Weapon };
use super::geometry::{ angle_from, get_nearest };


use rand::Rng;
use rand::distributions::Distribution;
use rand::seq::IteratorRandom;
use std::fmt::Write;
use super::brain::Brain as _;


const BARELY_CONNECTED: f32 = 0.001;
const UNCONNECTED: f32 = 0.0;


/// Process a fully connected layer
fn process_layer(neurons: &[Neuron], mut inputs: Vec<f32>) -> Vec<f32> {
    inputs.push(1.0);
    neurons.iter().map(|n| n.feed(&inputs)).collect()
}


fn unconnected_neuron(synapse_count: u8) -> Neuron {
    Neuron {
        weights: (0..synapse_count + 1).map(|_| UNCONNECTED).collect(),
        activation: Function::Linear,
    }
}

/// Does as little as possible while staying fully connected.
fn dumb_neuron(synapse_count: u8) -> Neuron {
    Neuron {
        weights: (0..synapse_count + 1).map(|_| BARELY_CONNECTED).collect(),
        activation: Function::Linear,
    }
}


fn dumb_hidden_layer(num_neurons: u8, input_count: u8) -> Vec<Neuron> {
    (0..input_count)
        .map(|_| dumb_neuron(INPUT_COUNT))
        .chain({
            (input_count..num_neurons)
                .map(|_| unconnected_neuron(INPUT_COUNT))
        })
        .collect()
}



fn dumb_output_layer(num_outputs: usize, synapse_count: u8) -> Vec<Neuron> {
    (0..num_outputs)
        .map(|i| {
            // Connect each neuron with the one directly "above" it.
            // It leaves the "overflow" of hidden neurons unconnected.
            let mut n = unconnected_neuron(synapse_count);
            n.weights[i] = BARELY_CONNECTED;
            n
        })
        .collect()
}

/// Brain used by the last stand hero
/// Uses a single hidden layer of neurons
#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    hidden_layer: Vec<Neuron>,
    output_layer: Vec<Neuron>,
    // TODO: remove
    mut_count: u16,
}

impl Brain {
    pub fn new_dumb(hidden_neurons: u8) -> Brain {
        Brain {
            hidden_layer: dumb_hidden_layer(hidden_neurons, INPUT_COUNT),
            output_layer: dumb_output_layer(2, hidden_neurons),
            mut_count: 0,
        }
    }
    
    pub fn pretty_print(&self) -> Result<String, fmt::Error> {
        let mut f = String::new();
        fn fmt_neurons(layer: &[Neuron], f: &mut String) -> fmt::Result {
            for neuron in layer {
                write!(f, "    {:?}: ", neuron.activation)?;
                for weight in &neuron.weights {
                    write!(f, "{:.3} ", weight)?;
                }
                write!(f, "\n")?;
            }
            Ok(())
        }
        writeln!(f, "Mut {}", self.mut_count)?;
        writeln!(f, "Hidden")?;
        fmt_neurons(&self.hidden_layer, &mut f)?;
        writeln!(f, "Out")?;
        fmt_neurons(&self.output_layer, &mut f)?;
        Ok(f)
    }
    
    pub fn to_dot<W: io::Write>(&self, mut f: &mut W) -> Result<(), io::Error> {
        fn fmt_neurons<W: io::Write>(layer: &[Neuron], f: &mut W, name: &str, inputs: &str) -> io::Result<()> {
            let a = |func| {
                use Function::*;
                match func {
                    &Gaussian => "I",
                    &Linear => "/",
                    &Logistic => "S",
                    &Step01 => "L",
                    &ReLU => "v",
                    _ => "?",
                }
            };
                    
            for (i, neuron) in layer.iter().enumerate() {
                let name = format!("{}{}", name, i);
                writeln!(f, r#"    {0} [label="{0}\n{1}"]"#, name, a(&neuron.activation))?;
                for (i, weight) in neuron.weights.iter().enumerate() {
                    writeln!(f, r#"    {}{} -> {} [label="{:.3}"]"#, inputs, i, name, weight)?;
                }
            }
            Ok(())
        }
        fn fmt_rank<W: io::Write>(f: &mut W, names: &[String]) -> io::Result<()> {
            write!(f, "    {{ rank=same")?;
            for name in names {
                write!(f, " {}", name)?;
            }
            writeln!(f, " }}")?;
            Ok(())
        }
        writeln!(f, "Digraph Shooter {{")?;
        fn name_layer(count: usize, name: &str) -> Vec<String> {
            (0..(count + 1)).map(|n| format!("{}{}", name, n)).collect()
        }
        fmt_rank(&mut f, &name_layer(INPUT_COUNT as usize, "I"))?;
        fmt_rank(&mut f, &name_layer(self.hidden_layer.len(), "H"))?;
        fmt_rank(&mut f, &name_layer(self.output_layer.len(), "O"))?;
        fmt_neurons(&self.hidden_layer, &mut f, "H", "I")?;
        fmt_neurons(&self.output_layer, &mut f, "O", "H")?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl brain::Brain for Brain {
    type Inputs = Inputs;
    type Outputs = Outputs;
    fn process(&mut self, inputs: Inputs) -> Outputs {
        let inputs = vec![inputs.mob_rel_angle, inputs.time_survived, 1.0];
        let hidden = process_layer(&self.hidden_layer, inputs);
        let outputs = process_layer(&self.output_layer, hidden);
        Outputs {
            walk: false,
            turn: outputs[1],
            shoot: true,
            aim_rel_angle: outputs[0],
        }
    }

    fn mutate(mut self, strength: f64) -> Brain {
        let weight_deviation = 0.5;
        let weight_rate = 1.0;
        let weight_dist = Bernoulli::new(strength * weight_rate).unwrap();
        let connect_rate = 0.1;
        let disconnect_rate = 0.25;
        let connect_dist = Bernoulli::new(strength * connect_rate).unwrap();
        let disconnect_dist = Bernoulli::new(strength * disconnect_rate).unwrap();
        let activation_rate = 0.3;
        let activation_dist = Bernoulli::new(strength * activation_rate).unwrap();
        let activation_options = [Function::Linear, Function::Step01, Function::Gaussian, Function::ReLU, Function::Logistic];
        let mut rng = rand::thread_rng();

        let mut mutate_layer = |mut layer: &mut [Neuron]| {
            for mut neuron in layer {
                for mut weight in neuron.weights.iter_mut() {
                    *weight = match *weight {
                        0.0 => match rng.sample(&connect_dist) {
                            true => rng.sample::<f32, _>(StandardNormal) * weight_deviation,
                            false => 0.0,
                        },
                        weight => match rng.sample(&disconnect_dist) {
                            true => 0.0,
                            false => match rng.sample(&weight_dist) {
                                true => weight + rng.sample::<f32, _>(StandardNormal) * weight_deviation,
                                false => weight,
                            }
                        }
                    }
                }
                if rng.sample(&activation_dist) {
                    neuron.activation = activation_options.iter().choose(&mut rng).unwrap().clone();
                }
            }
        };

        mutate_layer(&mut self.hidden_layer);
        mutate_layer(&mut self.output_layer);
        self.mut_count += 1;
        self
    }
}

pub struct Inputs {
    //mob_distance: f32,
    mob_rel_angle: f32,
    time_survived: f32,
}

const INPUT_COUNT: u8 = 2;

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
    assets: Res<assets::Assets>,
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
            mob_rel_angle: rot / f32::consts::PI,
            time_survived: borg.time_alive,
        });
        // Apply outputs. Might be better to do this in a separate step.
        body.wake_up(true);
        body.angvel = (outputs.turn * borg.rotation_speed).min(borg.rotation_speed).max(-borg.rotation_speed);
        body.linvel = body.position.rotation.transform_vector(&Vector2::new(
            0.0,
            borg.speed * match outputs.walk {
                true => 1.0,
                false => 0.0,
            }
        ));
        let weapons = weapons.iter_mut().filter(|(_w, _t, parent)| parent.0 == entity);
        for (mut weapon, mut transform, _parent) in weapons {
            let abs_angle = body.position.rotation.angle() + outputs.aim_rel_angle * f32::consts::PI;
            transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), abs_angle);
            if outputs.shoot {
                weapon_trigger(&mut weapon, &transform, &mut commands, &asset_server, &assets, &audio_output);
            }
        }
    }
}


pub type Genotype = Brain;

/*
#[derive(Clone)]
pub struct Genotype {
    brain: Brain,
    /// Tracking "true" generation number
    mutation_rounds: u16,
}*/

/// Third iteration.
/// Let's experiment with keeping Adam and Eve as a regular genotype,
/// as opposed to a spawn rate.
/// It will bias Adam/Eve to breed more often in the beginning of training.
/// Remove all below average genotypes once the generation size is reached.
/// That becomes the new generation size.
#[derive(Debug)]
pub struct GenePool {
    /// Mapping: breeding genotype, spawn rate
    /// Spawn rate should be derived from objective success
    /// In this case, it's seconds of survival
    genotypes: Vec<(Genotype, f64)>,
    generation_size: usize,
    preserved_total: u64,
    generations_spawned: u64,
}

impl GenePool {
    pub fn new_eden() -> GenePool {
        GenePool {
            genotypes: vec![
                (Brain::new_dumb(3), 10.0), // High rate of initial breeding to Adam/Eve
            ],
            generation_size: 3,
            generations_spawned: 0,
            preserved_total: 0,
        }
    }

    pub fn spawn(&self) -> Genotype {
        // Best performers will get filtered out at generation swap,
        // favoring them here seems to lead to magnification of flukes:
        // they get repeated needlessly.
        // While fluke's spawn will get filtered out,
        // it fills out spaces that would have been used for genetic diversity.
        let index = (0..self.genotypes.len()).choose(&mut rand::thread_rng()).unwrap();
        println!("Spawn offspring of {}", index);
        self.genotypes
            .get(index)
            .map(|(genotype, chance)| genotype.clone())
            .unwrap()
            .mutate(0.125)
    }

    pub fn preserve(&mut self, genotype: Genotype, fitness: f64) {
        self.preserved_total += 1;
        println!("Preserving {}: {} (total {})", self.genotypes.len(), fitness, self.preserved_total);
        self.genotypes.push((genotype, fitness));
        // Newly preserved begin to give some chances for the old generation to breed more than once.
        // But don't blow up the gene pool at each generation.
        if self.genotypes.len() >= 2 * self.generation_size {
            self.generations_spawned += 1;
            // Skip one as a way for flukes to leave the system.
            // They won't have elevated spawn within a generation, but will stick to many generations otherwise.
            let mut candidates: Vec<_> = self.genotypes.iter().skip(1).map(|c| c.clone()).collect();
            let average = candidates.iter()
                .map(|(_, v)| *v)
                .sum::<f64>() / candidates.len() as f64;
            // Caution: new generation may score worse...
            println!("New generation {} scores at least {}!", self.generations_spawned, average);
            let new: Vec<_> = candidates.iter()
                .filter(|(_, score)| score >= &average)
                .map(|c| c.clone())
                .collect();
            if new.len() < 2 {
                println!("Losers. Reshuffling.");
                candidates.push((Brain::new_dumb(3), average));
                let new = candidates.iter().rev().map(|c| c.clone()).collect();
                self.genotypes = new;
            } else {
                self.generation_size = new.len();
                self.genotypes = new;
                println!("{} breeds", self.generation_size);
            }
        }
    }
}
