use std::rc::Rc;



use crate::prelude::*;

use super::{traversal_node::TraversalNode, traversal_step::TraversalStep};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct UpstreamEdge<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub edge: EdgeDescriptor<E>,
    pub step: Rc<TraversalStep<T, E, A>>,
    pub traversal_index: usize,
    pub step_index: usize,
    pub creates_cycle: bool,
    pub is_reentrant: bool,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> UpstreamEdge<T, E, A> {
    pub fn new(
        edge: EdgeDescriptor<E>,
        step: Rc<TraversalStep<T, E, A>>,
        step_index: usize,
        traversal_index: usize,
    ) -> Self {
        Self {
            edge,
            creates_cycle: false,
            is_reentrant: false,
            step,
            step_index,
            traversal_index,
        }
    }

    pub fn set_step_index(&self, step_index: usize) -> Self {
        Self {
            step_index,
            ..self.clone()
        }
    }

    pub fn set_traversal_index(&self, traversal_index: usize) -> Self {
        Self {
            traversal_index,
            ..self.clone()
        }
    }

    pub fn set_creates_cycle(&self, creates_cycle: bool) -> Self {
        Self {
            creates_cycle,
            ..self.clone()
        }
    }
    pub fn set_is_reentrant(&self, is_reentrant: bool) -> Self {
        Self {
            is_reentrant,
            ..self.clone()
        }
    }
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct TraversalEdge<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    edge: EdgeDescriptor<E>,
    creates_cycle: bool,
    is_reentrant: bool,
    step: Rc<TraversalStep<T, E, A>>,
    step_index: usize,
    traversal_index: usize,
    target: TraversalNode<T, E, A>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> TraversalEdge<T, E, A> {
    pub fn new(
        edge: EdgeDescriptor<E>,
        step: Rc<TraversalStep<T, E, A>>,
        target: TraversalNode<T, E, A>,
        step_index: usize,
        traversal_index: usize,
    ) -> Self {
        Self {
            edge,
            creates_cycle: false,
            is_reentrant: false,
            step,
            step_index,
            traversal_index,
            target,
        }
    }

    pub fn set_step_index(&self, step_index: usize) -> Self {
        Self {
            step_index,
            ..self.clone()
        }
    }

    pub fn set_traversal_index(&self, traversal_index: usize) -> Self {
        Self {
            traversal_index,
            ..self.clone()
        }
    }

    pub fn set_creates_cycle(&self, creates_cycle: bool) -> Self {
        Self {
            creates_cycle,
            ..self.clone()
        }
    }
    pub fn set_is_reentrant(&self, is_reentrant: bool) -> Self {
        Self {
            is_reentrant,
            ..self.clone()
        }
    }
}
