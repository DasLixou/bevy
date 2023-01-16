use hashbrown::HashMap;
use slotmap::{HopSlotMap, SecondaryMap};

use crate::{
    error::GraphError,
    graphs::{
        edge::Edge,
        keys::{EdgeIdx, NodeIdx},
        Graph, SimpleGraph,
    },
};

/// Implementation of a `SimpleGraph` which uses `HashMap<NodeIdx, EdgeIdx>` for adjacencies
///
/// `SimpleGraph`s can only hold one edge between two nodes and can't have edges between the same node
#[derive(Clone)]
pub struct SimpleMapGraph<N, E, const DIRECTED: bool> {
    nodes: HopSlotMap<NodeIdx, N>,
    edges: HopSlotMap<EdgeIdx, Edge<E>>,
    adjacencies: SecondaryMap<NodeIdx, HashMap<NodeIdx, EdgeIdx>>,
}

impl<N, E, const DIRECTED: bool> Graph<N, E> for SimpleMapGraph<N, E, DIRECTED> {
    fn new() -> Self {
        Self {
            nodes: HopSlotMap::with_key(),
            edges: HopSlotMap::with_key(),
            adjacencies: SecondaryMap::new(),
        }
    }

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
    fn get_node(&self, idx: NodeIdx) -> Result<&N, GraphError> {
        if self.nodes.contains_key(idx) {
            unsafe { Ok(self.get_node_unchecked(idx)) }
        } else {
            Err(GraphError::NodeIdxDoesntExist(idx))
        }
    }

    #[inline]
    unsafe fn get_node_unchecked(&self, idx: NodeIdx) -> &N {
        self.nodes.get_unchecked(idx)
    }

    #[inline]
    fn get_node_mut(&mut self, idx: NodeIdx) -> Result<&mut N, GraphError> {
        if self.nodes.contains_key(idx) {
            unsafe { Ok(self.get_node_unchecked_mut(idx)) }
        } else {
            Err(GraphError::NodeIdxDoesntExist(idx))
        }
    }

    #[inline]
    unsafe fn get_node_unchecked_mut(&mut self, idx: NodeIdx) -> &mut N {
        self.nodes.get_unchecked_mut(idx)
    }

    #[inline]
    fn has_node(&self, node: NodeIdx) -> bool {
        self.nodes.contains_key(node)
    }

    #[inline]
    fn get_edge(&self, edge: EdgeIdx) -> Result<&E, GraphError> {
        match self.edges.get(edge) {
            Some(e) => Ok(&e.data),
            None => Err(GraphError::EdgeIdxDoesntExist(edge)),
        }
    }

    #[inline]
    fn get_edge_mut(&mut self, edge: EdgeIdx) -> Result<&mut E, GraphError> {
        match self.edges.get_mut(edge) {
            Some(e) => Ok(&mut e.data),
            None => Err(GraphError::EdgeIdxDoesntExist(edge)),
        }
    }

    fn remove_edge(&mut self, edge: EdgeIdx) -> Result<E, GraphError> {
        if self.edges.contains_key(edge) {
            unsafe { Ok(self.remove_edge_unchecked(edge)) }
        } else {
            Err(GraphError::EdgeIdxDoesntExist(edge))
        }
    }

    #[inline]
    fn edges_between(&self, from: NodeIdx, to: NodeIdx) -> Result<Vec<EdgeIdx>, GraphError> {
        match self.edge_between(from, to) {
            Ok(Some(idx)) => Ok(vec![idx]),
            Ok(None) => Ok(vec![]),
            Err(e) => Err(e),
        }
    }

    #[inline]
    unsafe fn edges_between_unchecked(&self, from: NodeIdx, to: NodeIdx) -> Vec<EdgeIdx> {
        vec![self.edge_between_unchecked(from, to)]
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

    fn remove_node(&mut self, node: NodeIdx) -> Result<N, GraphError> {
        if DIRECTED {
            let mut edges = vec![];
            for (edge, data) in &self.edges {
                let (src, dst) = data.indices();
                if dst == node || src == node {
                    edges.push(edge);
                }
            }
            for edge in edges {
                unsafe {
                    // SAFETY: we know it must exist
                    self.remove_edge_unchecked(edge); // TODO: can we have a `remove_edges` function?
                }
            }
            match self.nodes.remove(node) {
                Some(n) => {
                    unsafe {
                        // SAFETY: it will exist.
                        self.adjacencies.remove(node).unwrap_unchecked();
                    }
                    Ok(n)
                }
                None => Err(GraphError::NodeIdxDoesntExist(node)),
            }
        } else {
            for (_, edge) in self.edges_of(node) {
                unsafe {
                    // SAFETY: we know it must exist
                    self.remove_edge_unchecked(edge); // TODO: can we have a `remove_edges` function?
                }
            }
            match self.nodes.remove(node) {
                Some(n) => {
                    unsafe {
                        // SAFETY: it will exist.
                        self.adjacencies.remove(node).unwrap_unchecked();
                    }
                    Ok(n)
                }
                None => Err(GraphError::NodeIdxDoesntExist(node)),
            }
        }
    }

    fn new_edge(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> Result<EdgeIdx, GraphError> {
        if DIRECTED {
            if from == to {
                Err(GraphError::EdgeBetweenSameNode(from))
            } else if let Some(from_edges) = self.adjacencies.get(from) {
                if from_edges.contains_key(&to) {
                    Err(GraphError::EdgeBetweenAlreadyExists(from, to))
                } else if self.has_node(to) {
                    unsafe { Ok(self.new_edge_unchecked(from, to, edge)) }
                } else {
                    Err(GraphError::NodeIdxDoesntExist(to))
                }
            } else {
                Err(GraphError::NodeIdxDoesntExist(from))
            }
        } else if from == to {
            Err(GraphError::EdgeBetweenSameNode(from))
        } else if let Some(node_edges) = self.adjacencies.get(from) {
            if node_edges.contains_key(&to) {
                Err(GraphError::EdgeBetweenAlreadyExists(from, to))
            } else if let Some(other_edges) = self.adjacencies.get(to) {
                if other_edges.contains_key(&from) {
                    Err(GraphError::EdgeBetweenAlreadyExists(from, to))
                } else {
                    unsafe { Ok(self.new_edge_unchecked(from, to, edge)) }
                }
            } else {
                Err(GraphError::NodeIdxDoesntExist(to))
            }
        } else {
            Err(GraphError::NodeIdxDoesntExist(from))
        }
    }

    unsafe fn new_edge_unchecked(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> EdgeIdx {
        if DIRECTED {
            let idx = self.edges.insert(Edge {
                src: from,
                dst: to,
                data: edge,
            });
            self.adjacencies
                .get_unchecked_mut(from)
                .insert_unique_unchecked(to, idx);
            idx
        } else {
            let idx = self.edges.insert(Edge {
                src: from,
                dst: to,
                data: edge,
            });
            self.adjacencies
                .get_unchecked_mut(from)
                .insert_unique_unchecked(to, idx);
            self.adjacencies
                .get_unchecked_mut(to)
                .insert_unique_unchecked(from, idx);
            idx
        }
    }

    unsafe fn remove_edge_unchecked(&mut self, edge: EdgeIdx) -> E {
        if DIRECTED {
            let (from, to) = self.edges.get_unchecked(edge).indices();
            self.adjacencies.get_unchecked_mut(from).remove(&to);
            self.edges.remove(edge).unwrap().data
        } else {
            let (node, other) = self.edges.get_unchecked(edge).indices();
            self.adjacencies.get_unchecked_mut(node).remove(&other);
            self.adjacencies.get_unchecked_mut(other).remove(&node);
            self.edges.remove(edge).unwrap().data
        }
    }
}

impl<N, E, const DIRECTED: bool> SimpleGraph<N, E> for SimpleMapGraph<N, E, DIRECTED> {
    fn edge_between(&self, from: NodeIdx, to: NodeIdx) -> Result<Option<EdgeIdx>, GraphError> {
        if let Some(from_edges) = self.adjacencies.get(from) {
            if from_edges.contains_key(&to) {
                unsafe { Ok(Some(self.edge_between_unchecked(from, to))) }
            } else {
                Ok(None)
            }
        } else {
            Err(GraphError::NodeIdxDoesntExist(from))
        }
    }

    unsafe fn edge_between_unchecked(&self, from: NodeIdx, to: NodeIdx) -> EdgeIdx {
        self.adjacencies
            .get_unchecked(from)
            .get(&to)
            .cloned()
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::simple_graph_tests;

    simple_graph_tests!(super::SimpleMapGraph);
}
