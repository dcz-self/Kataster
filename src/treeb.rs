/*! Tree brain, like HE-NEAT */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */
use crate::brain;
use crate::brain::Function;
use std::collections::HashSet;


use std::iter::FromIterator;


type NeuronIndex = usize;

/// Find limits on layers in random topologies, trying to favor deper placements.
fn find_depths(connections: &[&[NeuronIndex]]) -> Vec<Vec<NeuronIndex>> {
    // 0 is the output layer, 1 is the layer before.
    let mut not_deeper_than: Vec<Vec<usize>>
        = vec![(0..(connections.len())).collect()];
    for i in 0.. {
        let no_deeper_than_next = not_deeper_than[i].iter()
            .flat_map(|idx| connections[*idx])
            .map(|i| *i)
            .collect();
        not_deeper_than.push(no_deeper_than_next);
    }
    not_deeper_than
}


/// Find layers in random topologies, trying to favor deper placements.
fn find_layers(connections: &[&[NeuronIndex]]) -> Vec<HashSet<NeuronIndex>> {
    let depth_limits = find_depths(connections);
    let mut found: HashSet<NeuronIndex> = HashSet::new();
    // Remove already accounted for ones, starting from input layer.
    depth_limits.into_iter().rev()
        .map(|limited| {
            let new: HashSet<NeuronIndex>
                = HashSet::from_iter(limited.into_iter())
                    .difference(&found)
                    .map(|i| *i)
                    .collect();
            found.extend(new.iter());
            new
        })
        .collect()
}


/// ID within brain's node table
#[derive(Clone, Copy, Debug, PartialEq)]
struct Idx(usize);

#[derive(Clone)]
struct Neuron {
    synapses: Vec<(Idx, f32)>,
    activation: Function,
}

impl Neuron {
    fn new_blank() -> Neuron {
        Neuron {
            synapses: Vec::new(),
            activation: Function::Linear,
        }
    }

    fn feed(&self, signals: &[f32]) -> f32 {
        self.activation.apply(
            signals.into_iter()
                .zip(self.synapses.iter().map(|(_, w)| *w))
                .map(|(i, w)| i * w)
                .sum()
        )
    }
}

#[derive(Clone)]
enum Node {
    /// Index of the input
    Input(usize),
    /// Input only, equal to 1.
    Bias,
    /// Input value carried over
    MemoryRead(usize),
    /// Member of the hidden layer
    Hidden(Neuron),
    /// Member of the output layer
    Output(usize, Neuron),
    /// Output preserving the value for the next iteration
    MemoryWrite(usize),
}

#[derive(Clone)]
struct Digraph(Vec<Node>);

impl Digraph {
    fn depth_first_collect<R, F: Fn(Idx, &[R]) -> R>(&self, idx: Idx, f: &F, init: &R) -> R {
        let next = match &self.0[idx.0] {
            Node::Output(_, neuron) => Some(neuron),
            Node::Hidden(neuron) => Some(neuron),
            _ => None,
        };
        match next {
            Some(neuron) => f(
                idx,
                &neuron.synapses
                    .iter()
                    .map(|(c, _)| self.depth_first_collect(*c, f, &init))
                    .collect::<Vec<_>>()
            ),
            None => f(idx, &[]),
        }
    }

    /// Goes depth first until it finds the first true
    fn depth_first_visit<F: Fn(Idx) -> bool>(&self, idx: Idx, f: &F) -> bool {
        let next = match &self.0[idx.0] {
            Node::Output(_, neuron) => Some(neuron),
            Node::Hidden(neuron) => Some(neuron),
            _ => None,
        };
        match next {
            Some(neuron) => {
                neuron.synapses
                    .iter()
                    .find(|(c, _)| self.depth_first_visit(*c, f))
                    .is_some()
            },
            None => f(idx),
        }
    }
}


/// No separate neuron create/remove.
/// Lack of incoming connections constitutes removal.
/// Always ensures one unconnected hidden neuron, and one unconnected storage.
#[derive(Clone)]
struct Brain {
    nodes: Digraph,
    /// Stores memories. When memory nodes get disconnected,
    /// this may be shrunk accordingly.
    memories: Vec<f32>,
}


impl Brain {
    fn new_minimal(input_count: usize, output_count: usize) -> Brain {
        Brain {
            nodes: Digraph(
                (0..input_count)
                    .map(Node::Input)
                    .chain(vec![Node::Bias].into_iter())
                    // A hidden layer neuron to connect to if desired
                    .chain(vec![Node::Hidden(Neuron::new_blank())].into_iter())
                    .chain((0..output_count).map(|i| Node::Output(i, Neuron::new_blank())))
                    // A single memory cell to connect to if desired
                    .chain(vec![Node::MemoryWrite(0)].into_iter())
                    .collect()
            ),
            memories: Vec::new(),
        }
    }
    
    fn is_predecessor(&self, anchor: Idx, target: Idx) -> bool {
        self.nodes.depth_first_visit(
            anchor,
            &|i| i == target,
        )
    }
/*
    fn add_connection(&mut self, from: Idx, to: Idx) {
        if is_e
    }
*/
}/*

impl brain::Brain for Brain {
    type Inputs = Vec<f32>;
    type Outputs = Vec<f32>;
    fn process(&mut self, inputs: Self::Inputs) -> Self::Outputs {
        panic!()
    }
}
*/


#[cfg(test)]
mod tests {
    use super::*;

    fn basic_connection_graph() -> Digraph {
        use super::Node::*;
        Digraph(vec![
            Input(0),
            Output(
                0,
                Neuron {
                    activation: Function::Linear,
                    synapses: vec![(Idx(0), 0.0)],
                },
            )
        ])
    }

    #[test]
    fn depth_first() {
        let graph = basic_connection_graph();
        let collected = graph.depth_first_collect(
            Idx(1), // Output neuron,
            // collect all indices
            &|idx, acc: &[Vec<Idx>]| {
                let mut sum: Vec<Idx> = vec![idx];
                for a in acc {
                    sum.extend(a.iter())
                }
                sum
            },
            &Vec::new(),
        );
        assert_eq!(collected, vec![Idx(1), Idx(0)]);
    }
}
