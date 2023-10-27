use std::rc::Rc;

use crate::prelude::*;

use super::{history_item::HistoryItem, history_store::HistoryStore};

pub struct HistoryInterface<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub history: HistoryStore<T, E, A>,
    apply_finalized_blueprint:
        Rc<dyn Fn(FinalizedBlueprint<T, E>, A, Option<Rc<A>>) -> Result<(), GraphError>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> HistoryInterface<T, E, A> {
    pub fn new(
        history_store: HistoryStore<T, E, A>,
        apply_finalized_blueprint: Rc<
            dyn Fn(FinalizedBlueprint<T, E>, A, Option<Rc<A>>) -> Result<(), GraphError>,
        >,
    ) -> Self {
        Self {
            history: history_store,
            apply_finalized_blueprint,
        }
    }

    pub fn undo(&self, undo_action: A) {
        let undo_item = self.history.undo();
        if let Some(undo_item) = undo_item {
            (self.apply_finalized_blueprint)(
                undo_item.blueprint,
                undo_action,
                Some(undo_item.action_data),
            )
            .unwrap();
        } 
    }
    pub fn redo(&self, redo_action: A) {
        let redo_item = self.history.redo();
        if let Some(redo_item) = redo_item {
            (self.apply_finalized_blueprint)(
                redo_item.blueprint,
                redo_action,
                Some(redo_item.action_data),
            )
            .unwrap();
        } 
    }

    pub fn push(&self, item: HistoryItem<T, E, A>) {
        self.history.push(item);
    }
}
