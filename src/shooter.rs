/*! Last stander AI */
/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use super::brain;
use super::brain::{ Function, Neuron };


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
