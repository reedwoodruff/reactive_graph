use im::Vector;

use crate::prelude::GraphTraits;

use super::traversal_node::TraversalNode;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct TraversalStepResult<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub entry: TraversalNode<T, E, A>,
    pub endpoints: Vector<TraversalNode<T, E, A>>,
}
