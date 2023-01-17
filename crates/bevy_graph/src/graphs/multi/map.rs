use hashbrown::HashMap;
use slotmap::{HopSlotMap, SecondaryMap};

use crate::{
    error::GraphError,
    graphs::{
        adjacency_storage::AdjacencyStorage,
        edge::Edge,
        keys::{EdgeIdx, NodeIdx},
        Graph,
    },
    utils::vecset::VecSet,
};

/// Implementation of a `MultiGraph` which uses `HashMap<NodeIdx, Vec<EdgeIdx>>` for adjacencies
///
/// `MultiGraph`s can hold multiple edges between two nodes and edges between the same node
#[derive(Clone)]
pub struct MultiMapGraph<N, E, const DIRECTED: bool> {
    nodes: HopSlotMap<NodeIdx, N>,
    edges: HopSlotMap<EdgeIdx, Edge<E>>,
    adjacencies: SecondaryMap<NodeIdx, AdjacencyStorage<HashMap<NodeIdx, Vec<EdgeIdx>>>>,
}

impl<N, E, const DIRECTED: bool> Graph<N, E> for MultiMapGraph<N, E, DIRECTED> {
    fn new() -> Self {
        Self {
            nodes: HopSlotMap::with_key(),
            edges: HopSlotMap::with_key(),
            adjacencies: SecondaryMap::new(),
        }
    }

    #[inline]
    fn is_directed(&self) -> bool {
        DIRECTED
    }

    #[inline]
    fn is_multigraph(&self) -> bool {
        true
    }

    #[inline]
    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn add_node(&mut self, node: N) -> NodeIdx {
        let idx = self.nodes.insert(node);
        let storage = if DIRECTED {
            AdjacencyStorage::Directed(HashMap::new(), HashMap::new())
        } else {
            AdjacencyStorage::Undirected(HashMap::new())
        };
        self.adjacencies.insert(idx, storage);
        idx
    }

    fn try_add_edge(
        &mut self,
        src: NodeIdx,
        dst: NodeIdx,
        value: E,
    ) -> Result<EdgeIdx, GraphError> {
        if !self.has_node(src) {
            Err(GraphError::NodeNotFound(src))
        } else if !self.has_node(dst) {
            Err(GraphError::NodeNotFound(dst))
        } else {
            unsafe {
                let idx = self.edges.insert(Edge(src, dst, value));
                self.adjacencies
                    .get_unchecked_mut(src)
                    .outgoing_mut()
                    .entry(dst)
                    .or_default()
                    .push(idx);
                self.adjacencies
                    .get_unchecked_mut(dst)
                    .incoming_mut()
                    .entry(src)
                    .or_default()
                    .push(idx);
                Ok(idx)
            }
        }
    }

    #[inline]
    fn has_node(&self, node: NodeIdx) -> bool {
        self.nodes.contains_key(node)
    }

    fn contains_edge_between(&self, src: NodeIdx, dst: NodeIdx) -> bool {
        self.adjacencies
            .get(src)
            .unwrap()
            .outgoing_mut()
            .contains_key(&dst)
    }

    fn remove_node(&mut self, index: NodeIdx) -> Option<N> {
        todo!()
    }

    fn remove_edge(&mut self, index: EdgeIdx) -> Option<E> {
        if let Some(Edge(src, dst, value)) = self.edges.remove(index) {
            unsafe {
                self.adjacencies
                    .get_unchecked_mut(src)
                    .outgoing_mut()
                    .get_mut(&dst)
                    .unwrap()
                    .remove_by_value(&index);
                self.adjacencies
                    .get_unchecked_mut(dst)
                    .incoming_mut()
                    .get_mut(&src)
                    .unwrap()
                    .remove_by_value(&index);
            }
            Some(value)
        } else {
            None
        }
    }

    fn clear_edges(&mut self) {
        self.adjacencies
            .values_mut()
            .for_each(|map| map.for_each_mut(HashMap::clear));
        self.edges.clear();
    }

    fn clear(&mut self) {
        self.adjacencies.clear();
        self.edges.clear();
        self.nodes.clear();
    }

    #[inline]
    fn get_node(&self, index: NodeIdx) -> Option<&N> {
        self.nodes.get(index)
    }

    #[inline]
    fn get_node_mut(&mut self, index: NodeIdx) -> Option<&mut N> {
        self.nodes.get_mut(index)
    }

    #[inline]
    fn get_edge(&self, index: EdgeIdx) -> Option<&E> {
        if let Some(Edge(_, _, value)) = self.edges.get(index) {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn get_edge_mut(&mut self, index: EdgeIdx) -> Option<&mut E> {
        if let Some(Edge(_, _, value)) = self.edges.get_mut(index) {
            Some(value)
        } else {
            None
        }
    }

    fn degree(&self, index: NodeIdx) -> usize {
        todo!()
    }

    type Nodes<'n> = slotmap::hop::Values<'n, NodeIdx, N> where Self: 'n;
    fn nodes(&self) -> Self::Nodes<'_> {
        self.nodes.values().into_iter()
    }

    type NodesMut<'n> = slotmap::hop::ValuesMut<'n, NodeIdx, N> where Self: 'n;
    fn nodes_mut(&mut self) -> Self::NodesMut<'_> {
        self.nodes.values_mut().into_iter()
    }
}
