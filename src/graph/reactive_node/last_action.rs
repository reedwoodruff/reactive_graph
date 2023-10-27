use std::rc::Rc;

use crate::prelude::{finalized_update_node::FinalizedUpdateNode, *};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct ActionData<A: GraphTraits> {
    pub main_action: A,
    pub secondary_action: Option<Rc<A>>,
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct LastAction<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub action_data: Rc<ActionData<A>>,
    pub update_info: Option<FinalizedUpdateNode<T, E>>,
}

impl<A: GraphTraits> ActionData<A> {
    pub fn new(action: A) -> Self {
        Self {
            main_action: action,
            secondary_action: None,
        }
    }
    pub fn set_secondary_action(&self, secondary_action: Rc<A>) -> Self {
        Self {
            secondary_action: Some(secondary_action),
            main_action: self.main_action.clone(),
        }
    }
}
