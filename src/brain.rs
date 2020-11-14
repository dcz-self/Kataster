/*! AI primitives */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

/// A generic brain
pub trait Brain {
    type Inputs;
    type Outputs;
    fn process(&mut self, inputs: Self::Inputs) -> Self::Outputs;
    /// Randomly alter itself, according to some abstract strength value
    fn mutate(self, strength: f64) -> Self;
}


#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Step01,
    StepNegPos,
    Linear,
    /// 1 / (1 + e ^ -x)
    Logistic,
    Tanh,
    ReLU,
    LReLu,
    Gaussian,
}

impl Function {
    pub fn apply(&self, value: f32) -> f32 {
        use Function::*;
        match self {
            Step01 => match value > 0.0 {
                true => 1.0,
                false => 0.0,
            },
            StepNegPos => match value > 0.0 {
                true => 1.0,
                false => -1.0,
            }
            Linear => value,
            Logistic => 1.0 / (1.0 + (-value).exp()),
            Tanh => value.tanh(),
            ReLU => match value > 0.0 {
                true => value,
                false => 0.0,
            },
            LReLu => match value > 0.0 {
                true => value,
                false => value * 0.01,
            },
            Gaussian => (-(value * value)).exp(),
        }
    }
}



/// Basic neuron. Bias is an input.
#[derive(Debug, Clone, PartialEq)]
pub struct Neuron {
    pub weights: Vec<f32>,
    pub activation: Function,
}

impl Neuron {
    pub fn feed(&self, inputs: &[f32]) -> f32 {
        self.activation.apply(
            inputs.into_iter()
                .zip(self.weights.iter())
                .map(|(i, w)| i * w)
                .sum()
        )
    }
}
