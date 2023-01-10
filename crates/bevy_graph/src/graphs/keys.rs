use slotmap::new_key_type;

use super::Graph;

new_key_type! {
    pub struct NodeIdx;
    pub struct EdgeIdx;
}

impl EdgeIdx {
    #[inline]
    pub fn get<N, E>(self, graph: &impl Graph<N, E>) -> Option<&E> {
        graph.get_edge(self)
    }

    #[inline]
    pub fn get_mut<N, E>(self, graph: &mut impl Graph<N, E>) -> Option<&mut E> {
        graph.get_edge_mut(self)
    }
}
