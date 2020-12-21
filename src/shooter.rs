/*! Last stander AI */
/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::app::Events;
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
use rand::distributions::{ Bernoulli, WeightedIndex };
use rand_distr::{ Binomial, StandardNormal };
use std::f32;
use std::fmt;
use std::io;
use super::assets;
use super::brain;
use super::brain::{ Function, Neuron };
use super::components::{ weapon_trigger, AttachedToEntity, Borg, LooksAt, Mob, Weapon };
use super::geometry::{ angle_from, get_nearest };


use crate::brain::MixableGenotype;
use rand::Rng;
use rand::seq::IteratorRandom;
use rand_distr::Distribution;
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


fn dumb_hidden_layer(num_neurons: u8, output_count: u8) -> Vec<Neuron> {
    (0..output_count)
        .map(|_| dumb_neuron(INPUT_COUNT))
        .chain({
            (output_count..num_neurons)
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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct NodeId(pub usize);

#[derive(Debug, PartialEq)]
pub enum Signal {
    Synapse {
        value: f32,
        from: NodeId,
        to: NodeId,
    },
    Neuron {
        raw_value: f32,
        activation_value: f32,
        id: NodeId,
    },
    Input {
        value: f32,
        id: NodeId,
    },
}

/// Brain used by the last stand hero
/// Uses a single hidden layer of neurons
#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    // TODO: remove those pubs. They are needed for drawing, which should be here anyway.
    pub hidden_layer: Vec<Neuron>,
    pub output_layer: Vec<Neuron>,
    // TODO: remove
    mut_count: u16,
}

impl Brain {
    pub fn new_dumb(hidden_neurons: u8) -> Brain {
        Brain {
            hidden_layer: dumb_hidden_layer(hidden_neurons, 3),
            output_layer: dumb_output_layer(3, hidden_neurons),
            mut_count: 0,
        }
    }

    pub fn normalize_inputs(inputs: Inputs) -> Vec<f32> {
        vec![inputs.mob_rel_angle, inputs.time_survived]
    }

    pub fn get_layers(&self) -> Vec<Vec<NodeId>> {
        let mut out: Vec<Vec<NodeId>> = vec![
            (0..(INPUT_COUNT as usize + 1)).map(NodeId).collect(),
            (0..(self.hidden_layer.len() + 1)).map(NodeId).collect(),
            (0..(self.output_layer.len())).map(NodeId).collect(),
        ];
        let mut prev = 0;
        for layer in out.iter_mut() {
            *layer = layer.into_iter().map(|id| NodeId(id.0 + prev)).collect();
            prev += layer.len()
        }
        out
    }

    pub fn get_node_layers(&self) -> Vec<(NodeId, u8)> {
        (0..(INPUT_COUNT + 1)).map(|_| 0)
            .chain((0..(self.hidden_layer.len() + 1)).map(|_| 1))
            .chain((0..(self.output_layer.len())).map(|_| 2))
            .enumerate()
            .map(|(node, layer)| (NodeId(node), layer))
            .collect()
    }

    pub fn find_signals(&self, inputs: Inputs) -> Vec<Signal> {
        let mut inputs = Brain::normalize_inputs(inputs);
        
        let layer_signals = |layer: &[Neuron], inputs: &[f32], input_id_offset, id_offset| {
            let mut signals = Vec::new();
            let mut outs = Vec::new();
            for (nidx, neuron) in layer.iter().enumerate() {
                let self_id = NodeId(nidx + id_offset);
                let values: Vec<_> = neuron.weights.iter().zip(inputs).map(|(w, i)| w * i).collect();
                for ((sidx, value), weight) in values.iter().enumerate().zip(neuron.weights.iter()) {
                    if *weight != 0.0 {
                        signals.push(Signal::Synapse {
                            value: *value,
                            from: NodeId(sidx + input_id_offset),
                            to: self_id,
                        });
                    }
                }
                let raw_value = values.into_iter().sum();
                let activation_value = neuron.activation.apply(raw_value);
                signals.push(Signal::Neuron {
                    raw_value,
                    activation_value,
                    id: self_id,
                });
                outs.push(activation_value);
            }
            (signals, outs)
        };
        let input_signals: Vec<Signal> = inputs.iter().enumerate()
            .map(|(i, value)| Signal::Input {
                id: NodeId(i),
                value: *value,
            }).collect();

        inputs.push(1.0);
        let in_offset = 0;
        let layer_offset = in_offset + inputs.len();
        let (hidden_signals, mut outs)
            = layer_signals(&self.hidden_layer, &inputs, in_offset, layer_offset);

        outs.push(1.0);
        let in_offset = layer_offset;
        let layer_offset = in_offset + outs.len();
        let (output_signals, _)
            = layer_signals(&self.output_layer, &outs, in_offset, layer_offset);

        input_signals.into_iter()
            .chain(hidden_signals.into_iter())
            .chain(output_signals.into_iter())
            .collect()
        /*
        Gen::new(|co| async move {
            
    co.yield_(self.mut_count).await;
    co.yield_(20).await;
})*/
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
        let inputs = Brain::normalize_inputs(inputs);
        let hidden = process_layer(&self.hidden_layer, inputs);
        let outputs = process_layer(&self.output_layer, hidden);
        Outputs {
            walk: outputs[2],
            turn: outputs[1],
            shoot: true,
            aim_rel_angle: outputs[0],
        }
    }

    fn mutate(mut self, strength: f64) -> Brain {
        let weight_deviation = 0.5;
        let weight_rate = 1.0;
        let weight_dist = Bernoulli::new(strength * weight_rate).unwrap();
        let connect_rate = 0.15;
        let disconnect_rate = 0.25;
        let connect_dist = Bernoulli::new(strength * connect_rate).unwrap();
        let disconnect_dist = Bernoulli::new(strength * disconnect_rate).unwrap();
        let activation_rate = 0.4;
        let activation_dist = Bernoulli::new(strength * activation_rate).unwrap();
        let activation_options = [Function::Linear, Function::Step01, Function::Gaussian, Function::ReLU, Function::Logistic];
        let mut rng = rand::thread_rng();

        let mut mutate_layer = |layer: &mut [Neuron]| {
            for mut neuron in layer {
                for weight in neuron.weights.iter_mut() {
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

impl brain::MixableGenotype for Brain {
    /// Mix by randomly choosing gene supplier.
    fn mix_with(&self, other: &Brain) -> Brain {
        let mut rng = rand::thread_rng();
        let parent_dist = Bernoulli::new(0.5).unwrap();

        let mut mix_neuron = |n0: &Neuron, n1: &Neuron| {
            Neuron {
                weights: {
                    n0.weights.iter()
                        .zip(n1.weights.iter())
                        .map(|(w0, w1)| *match parent_dist.sample(&mut rng) {
                            true => w0,
                            false => w1,
                        })
                        .collect()
                },
                activation: match parent_dist.sample(&mut rng) {
                    true => n0.activation.clone(),
                    false => n1.activation.clone(),
                },
            }
        };

        let mut mix_layer = |layer0: &[Neuron], layer1: &[Neuron]| {
            layer0.iter()
                .zip(layer1.iter())
                .map(|(n0, n1)| mix_neuron(n0, n1))
                .collect::<Vec<_>>()
        };
        
        Brain {
            hidden_layer: mix_layer(&self.hidden_layer, &other.hidden_layer),
            output_layer: mix_layer(&self.output_layer, &other.output_layer),
            mut_count: self.mut_count + other.mut_count,
            ..self.clone()
        }
    }
}

#[derive(Clone)]
pub struct Inputs {
    //mob_distance: f32,
    mob_rel_angle: f32,
    time_survived: f32,
}

const INPUT_COUNT: u8 = 2;

pub struct Outputs {
    walk: f32,
    /// Relative to walking direction
    turn: f32,
    shoot: bool,
    /// Relative to walking direction
    aim_rel_angle: f32,
}


pub struct BrainFed {
    pub entity: Entity,
    pub inputs: Inputs,
}

pub fn think(
    mut commands: Commands,
    mut brain_fed_events: ResMut<Events<BrainFed>>,
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
        .map(|body| body.position().translation.vector.clone().into())
        .collect();

    for (entity, body, borg, mut brain) in borgs.iter_mut() {
        let mut body = bodies.get_mut(body.handle()).unwrap();
        let nearest = get_nearest(&body.position().translation.vector.into(), &mob_positions)
            .unwrap_or(Point2::new(0.0, 0.0));
        let rot = angle_from(body.position(), &nearest);
        let inputs = Inputs {
            mob_rel_angle: rot / f32::consts::PI,
            time_survived: borg.time_alive,
        };
        brain_fed_events.send(BrainFed { entity, inputs: inputs.clone() });
        let outputs = brain.process(inputs);
        // Apply outputs. Might be better to do this in a separate step.
        body.set_angvel(
            (outputs.turn * borg.rotation_speed).min(borg.rotation_speed).max(-borg.rotation_speed),
            true,
        );
        body.set_linvel(
            body.position().rotation.transform_vector(&Vector2::new(
                0.0,
                borg.speed * outputs.walk.max(-1.0).min(1.0)
            )),
            true,
        );
        let weapons = weapons.iter_mut().filter(|(_w, _t, parent)| parent.0 == entity);
        for (mut weapon, mut transform, _parent) in weapons {
            let abs_angle = body.position().rotation.angle() + outputs.aim_rel_angle.max(-1.0).min(1.0) * f32::consts::PI;
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
    genotypes: Vec<(Genotype, f64, u64)>,
    preserved_total: u64,
}

impl GenePool {
    pub fn new_eden() -> GenePool {
        GenePool {
            genotypes: vec![
                // Let it be the main source of breeding
                // until reaching ideal population's fraction.
                // Expected total kills at population ten: 20. Be better than that.
                (Brain::new_dumb(3), 40.0 * 20.0, 0),
            ],
            preserved_total: 1,
        }
    }

    fn mutate(g: Genotype, times: u8, strength: f64) -> Genotype {
        if times == 0 {
            g
        } else {
            GenePool::mutate(g.mutate(strength), times - 1, strength)
        }
    }

    fn spawn_sexless(&self) -> Genotype {
        // Give them a chance to reflect their fitness.
        let distribution = WeightedIndex::new(
            self.genotypes.iter().map(|(_k, v, _id)| v + 40.0)
        ).unwrap();
        let index = distribution.sample(&mut rand::thread_rng());
        let (genotype, id) = self.genotypes
            .get(index)
            .map(|(genotype, _, id)| (genotype.clone(), id))
            .unwrap();
        println!("Spawn offspring of {}", id);
        GenePool::mutate(genotype, self.get_mut_rate(), 0.12)
    }

    fn get_mut_rate(&self) -> u8 {
        // Should probably be related to standard deviation of the population.
        // Instead just increase the rate when population is small.
        (20.0 / (self.genotypes.len() as f64 + 0.1)).ceil() as u8
    }
    
    /// Spawn hermaphoditic
    fn spawn_herm(&self) -> Genotype {
        let distribution = WeightedIndex::new(
            self.genotypes.iter().map(|(_k, v, _id)| v + 40.0)
        ).unwrap();
        let index0 = distribution.sample(&mut rand::thread_rng());
        let index1 = distribution.sample(&mut rand::thread_rng());
        
        let (genotype0, _w, id0) = self.genotypes.get(index0).unwrap();
        let (genotype1, _w, id1) = self.genotypes.get(index1).unwrap();
        println!("Spawn offspring of {} and {}", id0, id1);
        // Mutation rate shouldn't be too big;
        // there's enough mess due to sexual reproduction.
        GenePool::mutate(genotype0.mix_with(genotype1), self.get_mut_rate(), 0.06)
    }

    pub fn spawn(&self) -> Genotype {
        self.spawn_sexless()
    }

    pub fn preserve(&mut self, genotype: Genotype, fitness: f64) {
        self.genotypes.push((genotype, fitness, self.preserved_total));
        println!("Preserved as {} with score {}", self.preserved_total, fitness);
        println!("Pop {}", self.genotypes.len());
        self.preserved_total += 1;
        
        let ideal_pop_size = 20;
        let minimal_pop_size = ideal_pop_size / 4;
        let mut rng = rand::thread_rng();
        
        if self.genotypes.len() > ideal_pop_size * 2 / 3 {
            // Overpopulation. Remove oldies which already had a go.
            let dist = Binomial::new(
                self.genotypes.len() as u64,
                1.0 / (ideal_pop_size as f64),
            ).unwrap();
            let kill_count = dist.sample(&mut rng);
            let mut new: Vec<_>
                = self.genotypes.iter()
                    .skip(kill_count as usize)
                    .map(|c| c.clone())
                    .collect();
            println!("Killing {} oldies. Now pop {}.", kill_count, new.len());
            if new.len() < minimal_pop_size {
                println!("Filling up to {} with blanks", minimal_pop_size);
                new.resize(minimal_pop_size, (Brain::new_dumb(3), 40.0, 0));
            }
            self.genotypes = new;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signals() {
        let brain = Brain::new_dumb(3);
        let signals = brain.find_signals(Inputs {
            mob_rel_angle: 0.0,
            time_survived: 0.0,
        });
        //assert_eq!(signals, vec![]);
    }
}
