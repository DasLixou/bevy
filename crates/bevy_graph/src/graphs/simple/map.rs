use hashbrown::HashMap;
use slotmap::{HopSlotMap, Key, SecondaryMap};

use crate::{
    error::{GraphError, GraphResult},
    graphs::{
        edge::Edge,
        keys::{EdgeIdx, NodeIdx},
    },
    impl_graph,
};

#[derive(Clone)]
pub struct SimpleMapGraph<N, E, const DIRECTED: bool> {
    nodes: HopSlotMap<NodeIdx, N>,
    edges: HopSlotMap<EdgeIdx, Edge<E>>,
    adjacencies: SecondaryMap<NodeIdx, HashMap<NodeIdx, EdgeIdx>>,
}

impl<N, E, const DIRECTED: bool> SimpleMapGraph<N, E, DIRECTED> {
    pub fn new() -> Self {
        Self {
            nodes: HopSlotMap::with_key(),
            edges: HopSlotMap::with_key(),
            adjacencies: SecondaryMap::new(),
        }
    }
}

impl_graph! {
    impl common for SimpleMapGraph {
        #[inline]
        fn count(&self) -> usize {
            self.nodes.len()
        }

        #[inline]
        fn new_node(&mut self, node: N) -> NodeIdx {
            let idx = self.nodes.insert(node);
            self.adjacencies.insert(idx, HashMap::new());
            idx
        }

        #[inline]
        fn get_node(&self, idx: NodeIdx) -> GraphResult<&N> {
            if let Some(node) = self.nodes.get(idx) {
                Ok(node)
            } else {
                Err(GraphError::NodeDoesntExist(idx))
            }
        }

        #[inline]
        fn get_node_mut(&mut self, idx: NodeIdx) -> GraphResult<&mut N> {
            if let Some(node) = self.nodes.get_mut(idx) {
                Ok(node)
            } else {
                Err(GraphError::NodeDoesntExist(idx))
            }
        }

        #[inline]
        fn has_node(&self, node: NodeIdx) -> bool {
            self.nodes.contains_key(node)
        }

        #[inline]
        fn get_edge(&self, edge: EdgeIdx) -> GraphResult<&E> {
            match self.edges.get(edge) {
                Some(e) => Ok(&e.data),
                None => Err(GraphError::EdgeDoesntExist(edge))
            }
        }

        #[inline]
        fn get_edge_mut(&mut self, edge: EdgeIdx) -> GraphResult<&mut E> {
            match self.edges.get_mut(edge) {
                Some(e) => Ok(&mut e.data),
                None => Err(GraphError::EdgeDoesntExist(edge))
            }
        }

        #[inline]
        fn edge_between(&self, from: NodeIdx, to: NodeIdx) -> EdgeIdx {
            self.adjacencies
                .get(from)
                .unwrap()
                .get(&to)
                .cloned()
                .unwrap_or_else(EdgeIdx::null)
        }

        #[inline]
        fn edges_of(&self, node: NodeIdx) -> Vec<(NodeIdx, EdgeIdx)> {
            self.adjacencies
                .get(node)
                .unwrap()
                .iter()
                .map(|(node, edge)| (*node, *edge))
                .collect()
        }
    }

    impl undirected {
        fn remove_node(&mut self, node: NodeIdx) -> GraphResult<N> {
            for (_, edge) in self.edges_of(node) {
                self.remove_edge(edge).unwrap();
            }
            match self.nodes.remove(node) {
                Some(n) => Ok(n),
                None => Err(GraphError::NodeDoesntExist(node))
            }
        }

        fn new_edge(&mut self, node: NodeIdx, other: NodeIdx, edge: E) -> GraphResult<EdgeIdx> {
            if let Some(node_edges) = self.adjacencies.get(node) {
                if node_edges.contains_key(&other) {
                    Err(GraphError::EdgeAlreadyExists(node, other))
                } else if let Some(other_edges) = self.adjacencies.get(other) {
                    if other_edges.contains_key(&node) {
                        Err(GraphError::EdgeAlreadyExists(node, other))
                    } else {
                        unsafe {
                            Ok(self.new_edge_unchecked(node, other, edge))
                        }
                    }
                } else {
                    Err(GraphError::NodeDoesntExist(other))
                }
            } else {
                Err(GraphError::NodeDoesntExist(node))
            }
        }

        unsafe fn new_edge_unchecked(&mut self, node: NodeIdx, other: NodeIdx, edge: E) -> EdgeIdx {
            let idx = self.edges.insert(Edge {
                src: node,
                dst: other,
                data: edge,
            });
            self.adjacencies.get_unchecked_mut(node).insert_unique_unchecked(other, idx);
            self.adjacencies.get_unchecked_mut(other).insert_unique_unchecked(node, idx);
            idx
        }

        fn remove_edge(&mut self, edge: EdgeIdx) -> GraphResult<E> {
            if let Some((node, other)) = self.edges.get(edge).map(|e| e.indices()) {
                self.adjacencies.get_mut(node).unwrap().remove(&other);
                self.adjacencies.get_mut(other).unwrap().remove(&node);

                Ok(self.edges.remove(edge).unwrap().data)
            } else {
                Err(GraphError::EdgeDoesntExist(edge))
            }
        }
    }

    impl directed {
        fn remove_node(&mut self, node: NodeIdx) -> GraphResult<N> {
            let mut edges = vec![];
            for (edge, data) in &self.edges {
                let (src, dst) = data.indices();
                if dst == node || src == node {
                    edges.push(edge);
                }
            }
            for edge in edges {
                self.remove_edge(edge).unwrap();
            }
            match self.nodes.remove(node) {
                Some(n) => Ok(n),
                None => Err(GraphError::NodeDoesntExist(node))
            }
        }

        fn new_edge(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> GraphResult<EdgeIdx> {
            if let Some(from_edges) = self.adjacencies.get(from) {
                if from_edges.contains_key(&to) {
                    Err(GraphError::EdgeAlreadyExists(from, to))
                } else if self.has_node(to) {
                    unsafe {
                        Ok(self.new_edge_unchecked(from, to, edge))
                    }
                } else {
                    Err(GraphError::NodeDoesntExist(to))
                }
            } else {
                Err(GraphError::NodeDoesntExist(from))
            }
        }

        unsafe fn new_edge_unchecked(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> EdgeIdx {
            let idx = self.edges.insert(Edge {
                src: from,
                dst: to,
                data: edge,
            });
            self.adjacencies.get_unchecked_mut(from).insert_unique_unchecked(to, idx);
            idx
        }

        fn remove_edge(&mut self, edge: EdgeIdx) -> GraphResult<E> {
            if let Some((from, to)) = self.edges.get(edge).map(|e| e.indices()) {
                self.adjacencies.get_mut(from).unwrap().remove(&to);

                Ok(self.edges.remove(edge).unwrap().data)
            } else {
                Err(GraphError::EdgeDoesntExist(edge))
            }
        }
    }
}

impl<N, E, const DIRECTED: bool> Default for SimpleMapGraph<N, E, DIRECTED> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::graph_tests;

    graph_tests!(super::SimpleMapGraph);
}
