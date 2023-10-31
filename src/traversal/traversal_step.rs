use crate::prelude::{*};



#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum TraversalCount {
    /// Only returns the final node in a chain which is at least this long
    AtLeastExclusive(usize),
    /// Only returns every node in a chain which is at least this long
    AtLeastInclusive(usize),
    BetweenExclusive(usize, usize),
    BetweenInclusive(usize, usize),
    Exactly(usize),
}

impl TraversalCount {
    pub fn is_satisfied(&self, count: usize) -> bool {
        match self {
            TraversalCount::AtLeastInclusive(min) | TraversalCount::AtLeastExclusive(min) => {
                count >= *min
            }
            TraversalCount::BetweenExclusive(min, max)
            | TraversalCount::BetweenInclusive(min, max) => count >= *min && count <= *max,
            TraversalCount::Exactly(exact) => count == *exact,
        }
    }
    pub fn upper_bound_met(&self, count: usize) -> bool {
        match self {
            TraversalCount::AtLeastInclusive(_) | TraversalCount::AtLeastExclusive(_) => false,
            TraversalCount::BetweenExclusive(_, max) | TraversalCount::BetweenInclusive(_, max) => {
                count == *max
            }
            TraversalCount::Exactly(exact) => count == *exact,
        }
    }
    pub fn is_inclusive(&self) -> bool {
        match self {
            TraversalCount::AtLeastInclusive(_) | TraversalCount::BetweenInclusive(_, _) => true,
            _ => false,
        }
    }
    pub fn is_exclusive(&self) -> bool {
        match self {
            TraversalCount::AtLeastExclusive(_) | TraversalCount::BetweenExclusive(_, _) => true,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct TraversalStep<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub edge_finder: EdgeFinder<T, E, A>,
    pub count: TraversalCount,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> TraversalStep<T, E, A> {
    pub fn new(edge_finder: EdgeFinder<T, E, A>, count: TraversalCount) -> Self {
        Self { edge_finder, count }
    }
}

// fn get_target_node_if_matches<T: GraphTraits, E: GraphTraits, A: GraphTraits>(
//     edge: &EdgeDescriptor<E>,
//     edge_finder: &EdgeFinder<T, E, A>,
//     get_node_closure: GetNodeClosure<T, E, A>,
// ) -> Option<ReadReactiveNode<T, E, A>> {
//     if edge_finder.matches(edge) {
//         Some(edge.target)
//     } else {
//         None
//     }
// }
