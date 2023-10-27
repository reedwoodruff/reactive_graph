use std::rc::Rc;

use crate::prelude::{FinalizedBlueprint, GraphTraits};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoryItem<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub blueprint: FinalizedBlueprint<T, E>,
    pub action_data: Rc<A>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> HistoryItem<T, E, A> {
    pub fn history_invert(&self) -> Self {
        Self {
            blueprint: self.blueprint.invert_blueprint(),
            action_data: self.action_data.clone(),
        }
    }
}
