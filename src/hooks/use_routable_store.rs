use std::{cell::RefCell, rc::Rc};

use im::Vector;
use leptos::*;

use crate::prelude::{view_graph::ViewGraph, AllowedRenderEdgeSpecifier, GraphTraits};

#[derive(Clone, Debug)]
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

pub struct GraphSettings<E: GraphTraits> {
    pub render_edge_types: Option<Vector<AllowedRenderEdgeSpecifier<E>>>,
}

pub fn use_routable_store<T: GraphTraits, E: GraphTraits>(
    render_edge_types: Option<impl IntoIterator<Item = AllowedRenderEdgeSpecifier<E>>>,
) {
    provide_context::<Rc<RefCell<ViewGraph<T, E>>>>(Rc::new(
        RefCell::new(ViewGraph::<T, E>::new()),
    ));
    let (is_locked, set_is_locked) = create_signal(false);
    provide_context::<Rc<GraphLock>>(Rc::new(GraphLock {
        is_locked,
        set_is_locked,
    }));
    provide_context::<Rc<GraphSettings<E>>>(Rc::new(GraphSettings {
        render_edge_types: render_edge_types.map(|i| i.into_iter().collect()),
    }));
}
