use hashbrown::HashMap;
use slotmap::{HopSlotMap, SecondaryMap};

use crate::{
    error::{GraphError, GraphResult},
    graphs::{
        edge::Edge,
        keys::{EdgeIdx, NodeIdx},
    },
    impl_graph,
};

#[derive(Clone)]
pub struct MultiMapGraph<N, E, const DIRECTED: bool> {
    nodes: HopSlotMap<NodeIdx, N>,
    edges: HopSlotMap<EdgeIdx, Edge<E>>,
    adjacencies: SecondaryMap<NodeIdx, HashMap<NodeIdx, Vec<EdgeIdx>>>,
}

impl<N, E, const DIRECTED: bool> MultiMapGraph<N, E, DIRECTED> {
    pub fn new() -> Self {
        Self {
            nodes: HopSlotMap::with_key(),
            edges: HopSlotMap::with_key(),
            adjacencies: SecondaryMap::new(),
        }
    }
}

impl_graph! {
    impl COMMON for MultiMapGraph {
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
            if self.nodes.contains_key(idx) {
                unsafe {
                    Ok(self.get_node_unchecked(idx))
                }
            } else {
                Err(GraphError::NodeIdxDoesntExist(idx))
            }
        }

        #[inline]
        unsafe fn get_node_unchecked(&self, idx: NodeIdx) -> &N {
            self.nodes.get_unchecked(idx)
        }

        #[inline]
        fn get_node_mut(&mut self, idx: NodeIdx) -> GraphResult<&mut N> {
            if self.nodes.contains_key(idx) {
                unsafe {
                    Ok(self.get_node_unchecked_mut(idx))
                }
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
        fn get_edge(&self, edge: EdgeIdx) -> GraphResult<&E> {
            match self.edges.get(edge) {
                Some(e) => Ok(&e.data),
                None => Err(GraphError::EdgeIdxDoesntExist(edge))
            }
        }

        #[inline]
        fn get_edge_mut(&mut self, edge: EdgeIdx) -> GraphResult<&mut E> {
            match self.edges.get_mut(edge) {
                Some(e) => Ok(&mut e.data),
                None => Err(GraphError::EdgeIdxDoesntExist(edge))
            }
        }

        #[inline]
        fn edges_between(&self, _from: NodeIdx, _to: NodeIdx) -> GraphResult<Vec<EdgeIdx>> {
            todo!()
        }

        #[inline]
        unsafe fn edges_between_unchecked(&self, _from: NodeIdx, _to: NodeIdx) -> Vec<EdgeIdx> {
            todo!()
        }

        #[inline]
        fn edges_of(&self, _node: NodeIdx) -> Vec<(NodeIdx, EdgeIdx)> {
            todo!()
        }
    }

    impl COMMON?undirected {
        fn remove_node(&mut self, _node: NodeIdx) -> GraphResult<N> {
            todo!()
        }

        fn new_edge(&mut self, _from: NodeIdx, _to: NodeIdx, _edge: E) -> GraphResult<EdgeIdx> {
            todo!()
        }

        unsafe fn new_edge_unchecked(&mut self, _from: NodeIdx, _to: NodeIdx, _edge: E) -> EdgeIdx {
            todo!()
        }

        fn remove_edge(&mut self, _edge: EdgeIdx) -> GraphResult<E> {
            todo!()
        }
    }

    impl COMMON?directed {
        fn remove_node(&mut self, _node: NodeIdx) -> GraphResult<N> {
            todo!()
        }

        fn new_edge(&mut self, _from: NodeIdx, _to: NodeIdx, _edge: E) -> GraphResult<EdgeIdx> {
            todo!()
        }

        unsafe fn new_edge_unchecked(&mut self, _from: NodeIdx, _to: NodeIdx, _edge: E) -> EdgeIdx {
            todo!()
        }

        fn remove_edge(&mut self, _edge: EdgeIdx) -> GraphResult<E> {
            todo!()
        }
    }
}

impl<N, E, const DIRECTED: bool> Default for MultiMapGraph<N, E, DIRECTED> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
