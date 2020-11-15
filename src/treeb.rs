/*! Tree brain, like HE-NEAT */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */
use crate::brain;
use crate::brain::Function;
use std::collections::HashSet;
use std::ops::{ Index, IndexMut };


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
struct Digraph(Vec<Option<Node>>);

impl Index<Idx> for Digraph {
    type Output = Node;
    fn index(&self, idx: Idx) -> &Node {
        self.0[idx.0].as_ref().expect("Index emptied")
    }
}

impl IndexMut<Idx> for Digraph {
    fn index_mut(&mut self, idx: Idx) -> &mut Node {
        self.0[idx.0].as_mut().expect("Index emptied")
    }
}

impl From<Vec<Node>> for Digraph {
    fn from(nodes: Vec<Node>) -> Digraph {
        Digraph(nodes.into_iter().map(Some).collect())
    }
}

impl Digraph {
    fn depth_first_collect<R, F: Fn(Idx, &[R]) -> R>(&self, idx: Idx, f: &F) -> R {
        let next = match &self[idx] {
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
            let next = match &self[idx] {
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
        let valid_source = match self[from] {
            Node::MemoryWrite(_, _) => false,
            Node::Output(_, _) => false,
            _ => true,
        };
        if valid_source {
            // This is early, but let's assume wrong targets don't happen often.
            if self.is_predecessor(from, to) {
                Err("Target connects to source")
            } else {
                let neuron = match &mut self[to] {
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

    fn remove_connection(&mut self, from: Idx, to: Idx) -> Result<(), &str> {
        let neuron = match &mut self[to] {
            Node::Input(_) => None,
            Node::MemoryRead(_) => None,
            Node::Bias => None,
            Node::Output(_, neuron) => Some(neuron),
            Node::Hidden(neuron) => Some(neuron),
            Node::MemoryWrite(_, neuron) => Some(neuron),
        };
        match neuron {
            Some(neuron) => {
                let index = neuron.synapses.iter()
                    .position(|(i, _)| *i == from);
                match index {
                    Some(index) => {
                        neuron.synapses.remove(index);
                        Ok(())
                    },
                    None => Err("Connection does not exist"),
                }
            },
            None => Err("Target doesn't do connections"),
        }
    }

    fn add(&mut self, node: Node) -> Idx {
        let node = Some(node);
        let index = (0..self.0.len())
            .find(|i| self.0[*i].is_none());
        Idx(match index {
            Some(i) => {
                self.0[i] = node;
                i
            },
            None => {
                self.0.push(node);
                self.0.len() - 1
            }
        })
    }

    fn remove(&mut self, target: Idx) {
        self.0[target.0] = None;
        use Node::*;
        for node in self.0.iter_mut() {
            let mut neuron = match node {
                Some(Output(_, neuron)) => Some(neuron),
                Some(Hidden(neuron)) => Some(neuron),
                Some(MemoryWrite(_, neuron)) => Some(neuron),
                _ => None,
            };
            if let Some(neuron) = neuron {
                let synapses = neuron.synapses.clone().into_iter()
                    .filter(|(i, weight)| *i == target)
                    .collect();
                neuron.synapses = synapses;
            }
        }
        for i in (0..self.0.len()).rev() {
            if let None = self.0[i] {
                self.0.pop();
            }
        }
    }

    fn enumerate(&self) -> impl Iterator<Item=(Idx, &Node)> {
        self.0.iter().enumerate()
            .filter_map(|(i, n)| n.as_ref().map(|n| (Idx(i), n)))
    }

    fn position<P: Fn(&Node)->bool>(&self, pred: P) -> Option<Idx> {
        self.enumerate().find(|(_, n)| pred(n)).map(|(i, _)| i)
    }
}


/// No separate neuron create/remove.
/// Lack of incoming connections constitutes removal.
/// Always ensures one unconnected hidden neuron, and one unconnected storage.
/// (Unconnected counts as no incoming connections.)
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
                    .map(Some)
                    .collect()
            ),
            memories: Vec::new(),
        }
    }
    
    
    /// Adds connection while managing brain invariant: keep extra nodes ready.
    fn add_connection(&mut self, from: Idx, to: Idx, weight: f32) -> Result<(), &str> {
        enum Action {
            AddHidden,
            AddMemory,
            Nothing,
        };
        use Action::*;
        let action = match self.nodes[to] {
            Node::Hidden(_) => Action::AddHidden,
            Node::MemoryWrite(_, _) => Action::AddMemory,
            _ => Action::Nothing,
        };
        // Need to split it here because we're going to mutate nodes.
        match action {
            AddHidden => {
                self.nodes.add(Node::Hidden(Neuron::new_blank()));
            },
            AddMemory => {
                let idx = self.memories.len();
                self.nodes.add(Node::MemoryRead(idx));
                self.nodes.add(Node::MemoryWrite(idx, Neuron::new_blank()));
            },
            Nothing => {},
        };
        self.nodes.add_connection(from, to, weight)
    }

    /// Removes neurons if needed to maintain brain invariant.
    fn remove_connection(&mut self, from: Idx, to: Idx) -> Result<(), &str> {
        let is_removable = |neuron: &Neuron| neuron.synapses.len() == 0;
        enum Action {
            RemoveHidden, // Idx unneeded, it's "to"
            RemoveMemory(Idx), // Index of corresponding MemoryRead
            Nothing,
        };
        use Action::*;
        let action = match &self.nodes[to] {
            Node::Hidden(neuron) => match is_removable(neuron) {
                true => RemoveHidden,
                false => Nothing,
            },
            Node::MemoryWrite(i, neuron) => match is_removable(neuron) {
                true => RemoveMemory(
                    self.nodes
                        .position(|n| match n {
                            Node::MemoryRead(read_idx) => read_idx == i,
                            _ => false,
                        })
                        .expect(&format!("No memory read corresponding to {}", i))
                ),
                false => Nothing,
            }
            _ => Nothing,
        };
        // Need to split the match to drop borrow of nodes
        let idx = match action {
            RemoveHidden => self.nodes.remove(to),
            RemoveMemory(read_idx) => {
                self.nodes.remove(to);
                self.nodes.remove(read_idx);
            },
            Nothing => {},
        };
        self.nodes.remove_connection(from, to)
    }
}

impl brain::Brain for Brain {
    type Inputs = Vec<f32>;
    type Outputs = Vec<f32>;
    fn process(&mut self, inputs: Self::Inputs) -> Self::Outputs {
        use Node::*;
        let end_idxs: Vec<_> = self.nodes
            .enumerate()
            .filter_map(|(i, n)| match n.is_end() {
                true => Some(i),
                false => None,
            }).collect();
        
        let values = end_idxs.clone().into_iter()
            .map(|endidx| self.nodes.depth_first_collect(
                endidx,
                &|i, vals| match &self.nodes[i] {
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
            .map(|i| &self.nodes[i])
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
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_matches;
    
    use crate::brain::Brain as _;

    fn basic_connection_graph() -> Digraph {
        use super::Node::*;
        Digraph::from(vec![
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
    fn double_remove_connection() {
        let mut graph = basic_connection_graph();
        assert_matches!(graph.remove_connection(Idx(0), Idx(1)), Ok(()));
        assert_matches!(graph.remove_connection(Idx(0), Idx(1)), Err("Connection does not exist"));
    }


    #[test]
    fn reverse_connect_fail() {
        use super::Node::*;
        let mut graph = Digraph::from(vec![
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
            nodes: Digraph::from(vec![
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
            nodes: Digraph::from(vec![
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
    fn no_synapses() {
        let mut brain = Brain {
            nodes: Digraph::from(vec![
                Node::Input(0),
                Node::Input(1),
                Node::Output(
                    0,
                    Neuron {
                        activation: Function::Linear,
                        synapses: vec![],
                    },
                ),
            ]),
            memories: Vec::new(),
        };
        assert_eq!(brain.process(vec![4.0, 5.0]), vec![0.0]);
    }

    #[test]
    fn memory_read() {
        let mut brain = Brain {
            nodes: Digraph::from(vec![
                Node::Input(0),
                Node::MemoryRead(0),
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
            memories: vec![5.0],
        };
        assert_eq!(brain.process(vec![4.0]), vec![-7.0]);
    }

    #[test]
    fn memory_write() {
        let mut brain = Brain {
            nodes: Digraph::from(vec![
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
