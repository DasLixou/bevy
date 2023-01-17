use std::collections::VecDeque;

use hashbrown::HashSet;

use crate::graphs::{keys::NodeIdx, Graph};

/// Implementation of the [`BFS` algorythm](https://www.geeksforgeeks.org/breadth-first-search-or-bfs-for-a-graph/)
///
/// when `d` is the distance between a node and the startnode,
/// it will evaluate every node with `d=1`, then continue with `d=2` and so on.
pub struct BreadthFirstSearch {
    queue: VecDeque<NodeIdx>,
    visited: HashSet<NodeIdx>,
}

impl BreadthFirstSearch {
    /// Creates a new `BreadthFirstSearch` with a start node
    pub fn new(start: NodeIdx) -> Self {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        visited.insert(start);
        queue.push_back(start);

        Self { queue, visited }
    }

    /// Creates a new `BreadthFirstSearch` with a start node and the count of nodes for capacity reserving
    pub fn with_capacity(start: NodeIdx, node_count: usize) -> Self {
        let mut queue = VecDeque::with_capacity(node_count);
        let mut visited = HashSet::with_capacity(node_count);

        visited.insert(start);
        queue.push_back(start);

        Self { queue, visited }
    }

    /// Gets an immutable reference to the value of the next node from the algorithm
    pub fn next<'g, N, E>(&mut self, graph: &'g impl Graph<N, E>) -> Option<&'g N> {
        if let Some(node) = self.queue.pop_front() {
            for (idx, _) in graph.edges_of(node) {
                if !self.visited.contains(&idx) {
                    self.visited.insert(idx);
                    self.queue.push_back(idx);
                }
            }
            Some(graph.get_node(node).unwrap())
        } else {
            None
        }
    }

    /// Gets a mutable reference to the value of the next node from the algorithm.
    pub fn next_mut<'g, N, E>(&mut self, graph: &'g mut impl Graph<N, E>) -> Option<&'g mut N> {
        if let Some(node) = self.queue.pop_front() {
            for (idx, _) in graph.edges_of(node) {
                if !self.visited.contains(&idx) {
                    self.visited.insert(idx);
                    self.queue.push_back(idx);
                }
            }
            Some(graph.get_node_mut(node).unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        algos::bfs::BreadthFirstSearch,
        graphs::{simple::SimpleMapGraph, Graph},
    };

    #[test]
    fn basic_imperative_bfs() {
        let mut graph = SimpleMapGraph::<i32, (), true>::new();

        let zero = graph.add_node(0);
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);

        graph.add_edge(zero, one, ());
        graph.add_edge(zero, two, ());
        graph.add_edge(one, two, ());
        graph.add_edge(two, zero, ());
        graph.add_edge(two, three, ());

        let elements = vec![0, 2, 1, 3];

        let mut counted_elements = Vec::with_capacity(4);

        let mut bfs = BreadthFirstSearch::with_capacity(zero, graph.node_count());
        while let Some(node) = bfs.next(&graph) {
            counted_elements.push(*node);
        }

        assert_eq!(elements, counted_elements);
    }
}
