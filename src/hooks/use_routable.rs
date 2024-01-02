use std::{fmt::Formatter, rc::Rc};

use crate::{
    history::history_interface::HistoryInterface,
    prelude::{new_node::TempId, reactive_node::read_reactive_node::ReadReactiveNode, *},
    traversal::traversal_descriptor::TraversalDescriptor,
};

use im::HashMap;
use leptos::*;

pub type GetNodeClosure<T, E, A> =
    Rc<dyn Fn(&Uid) -> Result<Rc<ReadReactiveNode<T, E, A>>, GraphError>>;

pub type ProcessBlueprintReturn = Result<HashMap<TempId, Uid>, GraphError>;
#[derive(Clone)]
pub struct UseRoutableReturn<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub get_node_closure: GetNodeClosure<T, E, A>,
    pub(super) process_blueprint_closure:
        Rc<dyn Fn(BuildBlueprint<T, E, A>, A) -> ProcessBlueprintReturn>,
    pub(super) initiate_graph_closure:
        Rc<dyn Fn(BuildBlueprint<T, E, A>, A, TempId) -> ProcessBlueprintReturn>,
    pub graph_lock: Rc<GraphLock>,
    pub history: Rc<HistoryInterface<T, E, A>>,
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
    pub fn get_node(&self, id: &Uid) -> Result<Rc<ReadReactiveNode<T, E, A>>, GraphError> {
        (self.get_node_closure)(id)
    }
    pub fn process_blueprint(
        &self,
        blueprint: BuildBlueprint<T, E, A>,
        action_data: A,
    ) -> ProcessBlueprintReturn {
        (self.process_blueprint_closure)(blueprint, action_data)
    }
    pub fn initiate_graph(
        &self,
        blueprint: BuildBlueprint<T, E, A>,
        action_data: A,
        entry_point_temp_id: TempId,
    ) -> ProcessBlueprintReturn {
        (self.initiate_graph_closure)(blueprint, action_data, entry_point_temp_id)
    }
    pub fn traverse_search(&self, start_id: Uid) -> TraversalDescriptor<T, E, A> {
        TraversalDescriptor::new(start_id, self.get_node_closure.clone())
    }
}
pub fn use_routable<T: GraphTraits, E: GraphTraits, A: GraphTraits>(// id: Uid,
) -> Rc<UseRoutableReturn<T, E, A>> {
    use_context::<Rc<UseRoutableReturn<T, E, A>>>().expect("Context should exist")
}
