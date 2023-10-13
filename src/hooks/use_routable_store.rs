use std::{cell::RefCell, rc::Rc};

use leptos::*;

use crate::prelude::{view_graph::ViewGraph, GraphTraits};

pub struct GraphLock {
    pub is_locked: ReadSignal<bool>,
    set_is_locked: WriteSignal<bool>,
}

impl GraphLock {
    pub fn lock(&self) {
        self.set_is_locked.set(true);
    }

    pub fn unlock(&self) {
        self.set_is_locked.set(false);
    }
}

pub fn use_routable_store<T: GraphTraits, E: GraphTraits>() {
    provide_context::<Rc<RefCell<ViewGraph<T, E>>>>(Rc::new(
        RefCell::new(ViewGraph::<T, E>::new()),
    ));
    let (is_locked, set_is_locked) = create_signal(false);
    provide_context::<Rc<GraphLock>>(Rc::new(GraphLock {
        is_locked,
        set_is_locked,
    }));
}
