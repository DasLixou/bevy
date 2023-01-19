use std::marker::PhantomData;

use super::{edge::EdgeRef, keys::EdgeIdx, Graph};

/// An iterator which converts `&EdgeIdx` to a `EdgeRef` of the graph
pub struct Edges<'g, N, E: 'g, G: Graph<N, E>, I: Iterator<Item = &'g EdgeIdx>> {
    graph: &'g G,
    inner: I,
    phantom: PhantomData<(N, E)>,
}

impl<'g, N, E: 'g, G: Graph<N, E>, I: Iterator<Item = &'g EdgeIdx>> Edges<'g, N, E, G, I> {
    /// Creates a new `Edges` iterator over a graph with the provided `inner` iterator
    pub fn new(inner: I, graph: &'g G) -> Self {
        Self {
            graph,
            inner,
            phantom: PhantomData,
        }
    }
}

impl<'g, N, E: 'g, G: Graph<N, E>, I: Iterator<Item = &'g EdgeIdx>> Iterator
    for Edges<'g, N, E, G, I>
{
    type Item = EdgeRef<'g, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.inner.next() {
            self.graph.get_edge(*index)
        } else {
            None
        }
    }
}
