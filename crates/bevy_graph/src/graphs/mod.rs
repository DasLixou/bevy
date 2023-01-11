pub mod simple;

pub mod edge;
pub mod keys;

use crate::algos::bfs::BreadthFirstSearch;
use crate::error::GraphResult;

use self::keys::{EdgeIdx, NodeIdx};

pub trait Graph<N, E>: NewNode<N> + GetNode<N> + NewEdge<E> + GetEdge<E> + EdgeUtils {
    #[inline]
    fn algo_bfs(&self, start: NodeIdx) -> BreadthFirstSearch {
        BreadthFirstSearch::new(start, self.len())
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait NewNode<N> {
    fn new_node(&mut self, node: N) -> NodeIdx;

    fn len(&self) -> usize;
}

pub trait GetNode<N> {
    fn node(&self, idx: NodeIdx) -> GraphResult<&N>;
    fn node_mut(&mut self, idx: NodeIdx) -> GraphResult<&mut N>;
}

pub trait NewEdge<E> {
    fn new_edge(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> EdgeIdx;

    fn remove_edge(&mut self, edge: EdgeIdx) -> GraphResult<E>;
}

pub trait GetEdge<E> {
    fn get_edge(&self, edge: EdgeIdx) -> Option<&E>;
    fn get_edge_mut(&mut self, edge: EdgeIdx) -> Option<&mut E>;
}

pub trait EdgeUtils {
    fn edge_between(&self, from: NodeIdx, to: NodeIdx) -> EdgeIdx;
    fn edges_of(&self, node: NodeIdx) -> Vec<(NodeIdx, EdgeIdx)>; // TODO: can we use other type than Vec? maybe directly iterator?
}
