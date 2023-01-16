use slotmap::{HopSlotMap, SecondaryMap};

use crate::{
    error::{GraphError, GraphResult},
    graphs::{
        edge::Edge,
        keys::{EdgeIdx, NodeIdx},
    },
    impl_graph,
};

/// Implementation of a MultiGraph which uses `Vec<(NodeIdx, Vec<EdgeIdx>)>` for adjacencies
///
/// MultiGraphs can hold multiple edges between two nodes and edges between the same node
#[derive(Clone)]
pub struct MultiListGraph<N, E, const DIRECTED: bool> {
    nodes: HopSlotMap<NodeIdx, N>,
    edges: HopSlotMap<EdgeIdx, Edge<E>>,
    adjacencies: SecondaryMap<NodeIdx, Vec<(NodeIdx, Vec<EdgeIdx>)>>,
}

impl_graph! {
    impl COMMON for MultiListGraph {
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
            self.adjacencies.insert(idx, Vec::new());
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

        fn remove_edge(&mut self, edge: EdgeIdx) -> GraphResult<E> {
            if self.edges.contains_key(edge) {
                unsafe {
                    Ok(self.remove_edge_unchecked(edge))
                }
            } else {
                Err(GraphError::EdgeIdxDoesntExist(edge))
            }
        }

        fn edges_between(&self, from: NodeIdx, to: NodeIdx) -> GraphResult<Vec<EdgeIdx>> {
            if self.has_node(from) {
                unsafe {
                    Ok(self.edges_between_unchecked(from, to))
                }
            } else {
                Err(GraphError::NodeIdxDoesntExist(from))
            }
        }

        unsafe fn edges_between_unchecked(&self, from: NodeIdx, to: NodeIdx) -> Vec<EdgeIdx> {
            find_edge_list(self.adjacencies.get_unchecked(from), to).cloned().unwrap_or_default()
        }

        fn edges_of(&self, node: NodeIdx) -> Vec<(NodeIdx, EdgeIdx)> {
            if let Some(list) = self.adjacencies.get(node) {
                // TODO: can this be done with iterators?
                let mut result = Vec::new();
                for (target, edges) in list {
                    for edge in edges {
                        result.push((*target, *edge));
                    }
                }
                result
            } else {
                Vec::new()
            }
        }
    }

    impl COMMON?undirected {
        fn remove_node(&mut self, node: NodeIdx) -> GraphResult<N> {
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
                },
                None => Err(GraphError::NodeIdxDoesntExist(node))
            }
        }

        fn new_edge(&mut self, node: NodeIdx, other: NodeIdx, edge: E) -> GraphResult<EdgeIdx> {
            if self.has_node(node) {
                if self.has_node(other) {
                    unsafe {
                        Ok(self.new_edge_unchecked(node, other, edge))
                    }
                } else {
                    Err(GraphError::NodeIdxDoesntExist(other))
                }
            } else {
                Err(GraphError::NodeIdxDoesntExist(node))
            }
        }

        unsafe fn new_edge_unchecked(&mut self, node: NodeIdx, other: NodeIdx, edge: E) -> EdgeIdx {
            let idx = self.edges.insert(Edge {
                src: node,
                dst: other,
                data: edge,
            });
            let adjs = self.adjacencies.get_unchecked_mut(node);
            if let Some(list) = find_edge_list_mut(adjs, other) {
                list.push(idx);
            } else {
                adjs.push((other, vec![idx]));
            }
            let adjs = self.adjacencies.get_unchecked_mut(other);
            if let Some(list) = find_edge_list_mut(adjs, node) {
                list.push(idx);
            } else {
                adjs.push((node, vec![idx]));
            }
            idx
        }

        unsafe fn remove_edge_unchecked(&mut self, edge: EdgeIdx) -> E {
            let (from, to) = self.edges.get_unchecked(edge).indices();

            let list = self.adjacencies.get_unchecked_mut(from);
            let list = find_edge_list_mut(list, to).unwrap();
            list.swap_remove(find_edge(list, edge).unwrap()); // TODO: remove or swap_remove ?

            let list = self.adjacencies.get_unchecked_mut(to);
            let list = find_edge_list_mut(list, from).unwrap();
            list.swap_remove(find_edge(list, edge).unwrap()); // TODO: remove or swap_remove ?

            self.edges.remove(edge).unwrap().data
        }
    }

    impl COMMON?directed {
        fn remove_node(&mut self, node: NodeIdx) -> GraphResult<N> {
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
                },
                None => Err(GraphError::NodeIdxDoesntExist(node))
            }
        }

        fn new_edge(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> GraphResult<EdgeIdx> {
            if self.has_node(from) {
                if self.has_node(to) {
                    unsafe {
                        Ok(self.new_edge_unchecked(from, to, edge))
                    }
                } else {
                    Err(GraphError::NodeIdxDoesntExist(to))
                }
            } else {
                Err(GraphError::NodeIdxDoesntExist(from))
            }
        }

        unsafe fn new_edge_unchecked(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> EdgeIdx {
            let idx = self.edges.insert(Edge {
                src: from,
                dst: to,
                data: edge,
            });
            let adjs = self.adjacencies.get_unchecked_mut(from);
            if let Some(list) = find_edge_list_mut(adjs, to) {
                list.push(idx);
            } else {
                adjs.push((to, vec![idx]));
            }
            idx
        }

        unsafe fn remove_edge_unchecked(&mut self, edge: EdgeIdx) -> E {
            let (from, to) = self.edges.get_unchecked(edge).indices();
            let list = self.adjacencies.get_unchecked_mut(from);
            let list = find_edge_list_mut(list, to).unwrap();
            list.swap_remove(find_edge(list, edge).unwrap()); // TODO: remove or swap_remove ?
            self.edges.remove(edge).unwrap().data
        }
    }
}

// Util function
#[inline]
fn find_edge_list(list: &[(NodeIdx, Vec<EdgeIdx>)], node: NodeIdx) -> Option<&Vec<EdgeIdx>> {
    match list.iter().find(|l| l.0 == node) {
        Some((_, l)) => Some(l),
        None => None,
    }
}
#[inline]
fn find_edge_list_mut(
    list: &mut [(NodeIdx, Vec<EdgeIdx>)],
    node: NodeIdx,
) -> Option<&mut Vec<EdgeIdx>> {
    match list.iter_mut().find(|l| l.0 == node) {
        Some((_, l)) => Some(l),
        None => None,
    }
}
#[inline]
fn find_edge(list: &[EdgeIdx], edge: EdgeIdx) -> Option<usize> {
    list.iter().position(|edge_idx| *edge_idx == edge)
}

#[cfg(test)]
mod test {
    use crate::multi_graph_tests;

    multi_graph_tests!(super::MultiListGraph);
}
