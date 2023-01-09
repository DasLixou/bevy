use hashbrown::HashMap;
use slotmap::{HopSlotMap, SecondaryMap};

use crate::{DirectedGraph, EdgeIdx, Graph, NodeIdx, UndirectedGraph};

pub struct SimpleMapGraph<N, E, const DIRECTED: bool> {
    nodes: HopSlotMap<NodeIdx, N>,
    edges: HopSlotMap<EdgeIdx, E>,
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

impl<N, E, const DIRECTED: bool> Graph<N, E> for SimpleMapGraph<N, E, DIRECTED> {
    fn new_node(&mut self, node: N) -> NodeIdx {
        let idx = self.nodes.insert(node);
        self.adjacencies.insert(idx, HashMap::new());
        idx
    }

    #[inline]
    fn node(&self, idx: NodeIdx) -> Option<&N> {
        self.nodes.get(idx)
    }

    #[inline]
    fn node_mut(&mut self, idx: NodeIdx) -> Option<&mut N> {
        self.nodes.get_mut(idx)
    }

    #[inline]
    fn edge_id_between(&self, from: NodeIdx, to: NodeIdx) -> Option<EdgeIdx> {
        self.adjacencies.get(from)?.get(&to).map(|idx| *idx)
    }

    #[inline]
    fn edge_by_id(&self, edge: EdgeIdx) -> Option<&E> {
        self.edges.get(edge)
    }

    #[inline]
    fn edge_by_id_mut(&mut self, edge: EdgeIdx) -> Option<&mut E> {
        self.edges.get_mut(edge)
    }
}

impl<N, E> UndirectedGraph<N, E> for SimpleMapGraph<N, E, false> {
    fn new_edge(&mut self, node: NodeIdx, other: NodeIdx, edge: E) -> EdgeIdx {
        let idx = self.edges.insert(edge);
        self.adjacencies.get_mut(node).unwrap().insert(other, idx);
        self.adjacencies.get_mut(other).unwrap().insert(node, idx);
        idx
    }

    fn remove_edge_between(&mut self, node: NodeIdx, other: NodeIdx) {
        let edge_idx = self
            .adjacencies
            .get_mut(node)
            .unwrap()
            .remove(&other)
            .unwrap();

        self.adjacencies.get_mut(other).unwrap().remove(&node);

        self.edges.remove(edge_idx);
    }
}

impl<N, E> DirectedGraph<N, E> for SimpleMapGraph<N, E, true> {
    fn new_edge(&mut self, from: NodeIdx, to: NodeIdx, edge: E) -> EdgeIdx {
        let idx = self.edges.insert(edge);
        self.adjacencies.get_mut(from).unwrap().insert(to, idx);
        idx
    }
    fn remove_edge_between(&mut self, from: NodeIdx, to: NodeIdx) {
        let edge_idx = self.adjacencies.get_mut(from).unwrap().remove(&to).unwrap();

        self.edges.remove(edge_idx);
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
    use crate::{DirectedGraph, Graph, UndirectedGraph};

    use super::SimpleMapGraph;

    enum Person {
        Jake,
        Michael,
        Jennifer,
    }

    #[test]
    fn undirected_edge() {
        const STRENGTH: i32 = 100;

        let mut map_graph = SimpleMapGraph::<Person, i32, false>::new();

        let jake = map_graph.new_node(Person::Jake);
        let michael = map_graph.new_node(Person::Michael);

        let _best_friends = map_graph.new_edge(jake, michael, STRENGTH);

        let strength_jake = map_graph.edge_between(jake, michael);
        assert!(strength_jake.is_some());
        assert_eq!(strength_jake.unwrap(), &STRENGTH);

        let strength_michael = map_graph.edge_between(michael, jake);
        assert!(strength_michael.is_some());
        assert_eq!(strength_michael.unwrap(), &STRENGTH);

        map_graph.remove_edge_between(michael, jake);

        let strength_jake = map_graph.edge_between(jake, michael);
        assert!(strength_jake.is_none());

        let strength_michael = map_graph.edge_between(michael, jake);
        assert!(strength_michael.is_none());
    }

    #[test]
    fn directed_edge() {
        const STRENGTH: i32 = 9999;

        let mut map_graph = SimpleMapGraph::<Person, i32, true>::new();

        let jake = map_graph.new_node(Person::Jake);
        let jennifer = map_graph.new_node(Person::Jennifer);

        let _oneway_crush = map_graph.new_edge(jake, jennifer, STRENGTH);

        let strength_jake = map_graph.edge_between(jake, jennifer);
        assert!(strength_jake.is_some());
        assert_eq!(strength_jake.unwrap(), &STRENGTH);

        let strength_jennifer = map_graph.edge_between(jennifer, jake);
        assert!(strength_jennifer.is_none());

        map_graph.remove_edge_between(jake, jennifer);

        let strength_jake = map_graph.edge_between(jake, jennifer);
        assert!(strength_jake.is_none());

        let strength_jennifer = map_graph.edge_between(jennifer, jake);
        assert!(strength_jennifer.is_none());
    }
}
