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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
    MemoryWrite(usize, Neuron),
}

impl Node {
    fn is_end(&self) -> bool {
        use Node::*;
        match self {
            Output(_, _) => true,
            MemoryWrite(_, _) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
struct Digraph(Vec<Node>);

impl Digraph {
    fn depth_first_collect<R, F: Fn(Idx, &[R]) -> R>(&self, idx: Idx, f: &F) -> R {
        let next = match &self.0[idx.0] {
            Node::Output(_, neuron) => Some(neuron),
            Node::Hidden(neuron) => Some(neuron),
            Node::MemoryWrite(_, neuron) => Some(neuron),
            _ => None,
        };
        match next {
            Some(neuron) => f(
                idx,
                &neuron.synapses
                    .iter()
                    .map(|(c, _)| self.depth_first_collect(*c, f))
                    .collect::<Vec<_>>()
            ),
            None => f(idx, &[]),
        }
    }

    /// Goes depth first until it finds the first true
    fn depth_first_visit<F: Fn(Idx) -> bool>(&self, idx: Idx, f: &F) -> bool {
        f(idx) || {
            let next = match &self.0[idx.0] {
                Node::Output(_, neuron) => Some(neuron),
                Node::Hidden(neuron) => Some(neuron),
                Node::MemoryWrite(_, neuron) => Some(neuron),
                _ => None,
            };
            match next {
                Some(neuron) => {
                    neuron.synapses
                        .iter()
                        .find(|(c, _)| self.depth_first_visit(*c, f))
                        .is_some()
                },
                None => false,
            }
        }
    }
    
    /// Return True is there is a path from target to anchor.
    fn is_predecessor(&self, anchor: Idx, target: Idx) -> bool {
        self.depth_first_visit(
            anchor,
            &|i| i == target,
        )
    }

    fn add_connection(&mut self, from: Idx, to: Idx, weight: f32) -> Result<(), &str> {
        let valid_source = match self.0[from.0] {
            Node::MemoryWrite(_, _) => false,
            Node::Output(_, _) => false,
            _ => true,
        };
        if valid_source {
            // This is early, but let's assume wrong targets don't happen often.
            if self.is_predecessor(from, to) {
                Err("Target connects to source")
            } else {
                let neuron = match &mut self.0[to.0] {
                    Node::Input(_) => None,
                    Node::MemoryRead(_) => None,
                    Node::Bias => None,
                    Node::Output(_, neuron) => Some(neuron),
                    Node::Hidden(neuron) => Some(neuron),
                    Node::MemoryWrite(_, neuron) => Some(neuron),
                };
                match neuron {
                    Some(neuron) => {
                        let exists = neuron.synapses.iter()
                            .find(|(i, _)| *i == from)
                            .is_some();
                        match exists {
                            false => Ok(neuron.synapses.push((from, weight))),
                            true => Err("Connection exists"),
                        }
                    },
                    None => Err("Invalid target type"),
                }
            }
        } else {
            Err("Invalid source")
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
                    .chain(vec![Node::MemoryWrite(0, Neuron::new_blank())].into_iter())
                    .collect()
            ),
            memories: Vec::new(),
        }
    }
}

impl brain::Brain for Brain {
    type Inputs = Vec<f32>;
    type Outputs = Vec<f32>;
    fn process(&mut self, inputs: Self::Inputs) -> Self::Outputs {
        use Node::*;
        let end_idxs: Vec<_> = self.nodes.0.iter()
            .enumerate()
            .filter_map(|(i, n)| match n.is_end() {
                true => Some(Idx(i)),
                false => None,
            }).collect();
        
        let values = end_idxs.clone().into_iter()
            .map(|endidx| self.nodes.depth_first_collect(
                endidx,
                &|i, vals| match &self.nodes.0[i.0] {
                    Bias => 1.0,
                    Hidden(neuron) => neuron.feed(vals),
                    Input(idx) => inputs[*idx],
                    MemoryRead(idx) => self.memories[*idx],
                    MemoryWrite(_, neuron) => neuron.feed(vals),
                    Output(_, neuron) => neuron.feed(vals),
                },
            ));
        
        let mut outputs = Vec::new();
        let mut memories: Vec<_>
            = (0..self.memories.len()).map(|_| 1337.0).collect();
        
        for (n, v) in end_idxs.into_iter()
            .map(|i| &self.nodes.0[i.0])
            .zip(values)
        {
            match n {
                Output(i, _) => {
                    if outputs.len() <= *i {
                        // Output array should not have holes,
                        // so if there are any left, they will be easily seen.
                        // I hope.
                        outputs.resize(*i + 1, 1337.0);
                    }
                    outputs[*i] = v;
                },
                MemoryWrite(i, _) => { memories[*i] = v; },
                node => println!("Invalid node in outputs: {:?}", node),
            };
        }

        // Could check for holes here.

        self.memories = memories;
        outputs
    }
    
    fn mutate(self, strength: f64) -> Self {
        panic!();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_matches;
    
    use crate::brain::Brain as _;

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
        );
        assert_eq!(collected, vec![Idx(1), Idx(0)]);
    }
    
    #[test]
    fn predecessor() {
        let graph = basic_connection_graph();
        assert_eq!(graph.is_predecessor(Idx(1), Idx(0)), true);
    }

    #[test]
    fn bad_connection() {
        let mut graph = basic_connection_graph();
        assert_matches!(graph.add_connection(Idx(1), Idx(0), 0.0), Err(_));
    }

    #[test]
    fn dupe_connection() {
        let mut graph = basic_connection_graph();
        assert_matches!(graph.add_connection(Idx(0), Idx(1), 0.0), Err("Connection exists"));
    }

    #[test]
    fn self_connection() {
        let mut graph = basic_connection_graph();
        assert_matches!(graph.add_connection(Idx(0), Idx(0), 0.0), Err("Target connects to source"));
    }

    #[test]
    fn reverse_connect_fail() {
        use super::Node::*;
        let mut graph = Digraph(vec![
            Hidden(Neuron {
                activation: Function::Linear,
                synapses: vec![],
            }),
            Hidden(Neuron {
                activation: Function::Linear,
                synapses: vec![(Idx(0), 0.0)],
            }),
        ]);
        assert_matches!(graph.add_connection(Idx(1), Idx(0), 0.0), Err("Target connects to source"));
    }
    
    #[test]
    fn bias() {
        let mut brain = Brain {
            nodes: Digraph(vec![
                Node::Bias,
                Node::Output(
                    0,
                    Neuron {
                        activation: Function::Linear,
                        synapses: vec![(Idx(0), 2.0)],
                    },
                ),
            ]),
            memories: Vec::new(),
        };
        assert_eq!(brain.process(Vec::new()), vec![2.0]);
    }

    #[test]
    fn inputs() {
        let mut brain = Brain {
            nodes: Digraph(vec![
                Node::Input(0),
                Node::Input(1),
                Node::Output(
                    0,
                    Neuron {
                        activation: Function::Linear,
                        synapses: vec![
                            (Idx(0), 2.0),
                            (Idx(1), -3.0),
                        ],
                    },
                ),
            ]),
            memories: Vec::new(),
        };
        assert_eq!(brain.process(vec![4.0, 5.0]), vec![-7.0]);
    }


    #[test]
    fn memory_read() {
        let mut brain = Brain {
            nodes: Digraph(vec![
                Node::Input(0),
                Node::MemoryRead(0), // should be 0 initially
                Node::Output(
                    0,
                    Neuron {
                        activation: Function::Linear,
                        synapses: vec![
                            (Idx(0), 2.0),
                            (Idx(1), -3.0),
                        ],
                    },
                ),
            ]),
            memories: vec![0.0],
        };
        assert_eq!(brain.process(vec![4.0, 5.0]), vec![8.0]);
    }

    #[test]
    fn memory_write() {
        let mut brain = Brain {
            nodes: Digraph(vec![
                Node::Input(0),
                Node::MemoryWrite(
                    0,
                    Neuron {
                        activation: Function::Linear,
                        synapses: vec![
                            (Idx(0), 2.0),
                        ],
                    },
                ),
            ]),
            memories: vec![0.0],
        };
        assert_eq!(brain.process(vec![4.0]), Vec::<f32>::new());
        assert_eq!(brain.memories, vec![8.0]);
    }
}
