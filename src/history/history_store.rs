use im::Vector;
use leptos::{create_signal, ReadSignal, SignalSet, SignalUpdate, WriteSignal};

pub use crate::prelude::*;

use super::history_item::HistoryItem;

pub struct HistoryStore<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub undo_stack: ReadSignal<Vector<HistoryItem<T, E, A>>>,
    pub redo_stack: ReadSignal<Vector<HistoryItem<T, E, A>>>,
    set_undo_stack: WriteSignal<Vector<HistoryItem<T, E, A>>>,
    set_redo_stack: WriteSignal<Vector<HistoryItem<T, E, A>>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Default for HistoryStore<T, E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> HistoryStore<T, E, A> {
    pub fn new() -> Self {
        let (undo_stack, set_undo_stack) = create_signal(Vector::new());
        let (redo_stack, set_redo_stack) = create_signal(Vector::new());
        Self {
            undo_stack,
            set_undo_stack,
            redo_stack,
            set_redo_stack,
        }
    }
    pub(super) fn push(&self, item: HistoryItem<T, E, A>) {
        self.set_undo_stack.update(|prev| prev.push_back(item));
        self.set_redo_stack.set(Vector::new());
    }
    pub(super) fn undo(&self) -> Option<HistoryItem<T, E, A>> {
        let item = self
            .set_undo_stack
            .try_update(|prev| prev.pop_back())??
            .history_invert();
        self.set_redo_stack
            .update(|prev| prev.push_back(item.clone()));
        Some(item)
    }
    pub(super) fn redo(&self) -> Option<HistoryItem<T, E, A>> {
        let item = self
            .set_redo_stack
            .try_update(|prev| prev.pop_back())??
            .history_invert();
        self.set_undo_stack
            .update(|prev| prev.push_back(item.clone()));
        Some(item)
    }
}
