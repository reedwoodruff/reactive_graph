use std::{cell::RefCell, fmt::Formatter, rc::Rc};

use crate::prelude::{
    new_node::TempId, reactive_node::read_reactive_node::ReadReactiveNode,
    utils::log_finalize_results, view_graph::ViewGraph, *,
};
use im::Vector;
use leptos::{logging::log, *};

#[derive(Clone)]
pub struct UseRoutableReturn<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub(super) get_node_closure:
        Rc<dyn Fn(Uid) -> Result<Rc<ReadReactiveNode<T, E, A>>, GraphError>>,
    pub(super) process_blueprint_closure:
        Rc<dyn Fn(BuildBlueprint<T, E>, A) -> Result<(), GraphError>>,
    pub(super) initiate_graph_closure:
        Rc<dyn Fn(BuildBlueprint<T, E>, A, TempId) -> Result<Uid, GraphError>>,
    pub graph_lock: Rc<GraphLock>,
}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> core::fmt::Debug
    for UseRoutableReturn<T, E, A>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseRoutableReturn")
            .field("graph_lock", &self.graph_lock)
            .finish()
    }
}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> UseRoutableReturn<T, E, A> {
    pub fn get_node(&self, id: Uid) -> Result<Rc<ReadReactiveNode<T, E, A>>, GraphError> {
        (self.get_node_closure)(id)
    }
    pub fn process_blueprint(
        &self,
        blueprint: BuildBlueprint<T, E>,
        action_data: A,
    ) -> Result<(), GraphError> {
        (self.process_blueprint_closure)(blueprint, action_data)
    }
    pub fn initiate_graph(
        &self,
        blueprint: BuildBlueprint<T, E>,
        action_data: A,
        entry_point_temp_id: TempId,
    ) -> Result<Uid, GraphError> {
        (self.initiate_graph_closure)(blueprint, action_data, entry_point_temp_id)
    }
}
pub fn use_routable<T: GraphTraits, E: GraphTraits, A: GraphTraits>(// id: Uid,
) -> Rc<UseRoutableReturn<T, E, A>> {
    use_context::<Rc<UseRoutableReturn<T, E, A>>>().expect("Context should exist")
}
