use std::rc::Rc;

use im::Vector;

use crate::prelude::{reactive_node::read_reactive_node::ReadReactiveNode, *};

use super::traversal_edge::{TraversalEdge, UpstreamEdge};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct TraversalNode<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub node: Rc<ReadReactiveNode<T, E, A>>,
    pub step_index: usize,
    pub traversal_index: usize,
    pub downstream_edges: Vector<TraversalEdge<T, E, A>>,
    pub upstream_edge: Option<UpstreamEdge<T, E, A>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> TraversalNode<T, E, A> {
    pub fn new(
        node: Rc<ReadReactiveNode<T, E, A>>,
        step_index: usize,
        traversal_index: usize,
    ) -> Self {
        Self {
            node,
            step_index,
            traversal_index,
            downstream_edges: Vector::new(),
            upstream_edge: None,
        }
    }

    pub fn add_downstream_edge(&self, edge: TraversalEdge<T, E, A>) -> Self {
        let mut new_downstream = self.downstream_edges.clone();
        new_downstream.push_back(edge);
        Self {
            downstream_edges: new_downstream,
            ..self.clone()
        }
    }
    pub fn set_upstream_edge(&self, edge: UpstreamEdge<T, E, A>) -> Self {
        Self {
            upstream_edge: Some(edge),
            ..self.clone()
        }
    }
    pub fn reset_step_index(&self) -> Self {
        Self {
            step_index: 0,
            ..self.clone()
        }
    }
}
