use std::{cell::RefCell, fmt::Formatter, rc::Rc};

use crate::prelude::{
    new_node::TempId, reactive_node::read_reactive_node::ReadReactiveNode,
    utils::log_finalize_results, view_graph::ViewGraph, *,
};
use im::Vector;
use leptos::{logging::log, *};

#[derive(Clone)]
pub struct UseRoutableReturn<T: GraphTraits, E: GraphTraits> {
    pub node: Rc<ReadReactiveNode<T, E>>,
    pub view_graph: Rc<RefCell<ViewGraph<T, E>>>,
    pub process_blueprint:
        Rc<dyn Fn(BuildBlueprint<T, E>, Option<TempId>) -> Result<(), GraphError>>,
    pub graph_lock: Rc<GraphLock>,
}
impl<T: GraphTraits, E: GraphTraits> core::fmt::Debug for UseRoutableReturn<T, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UseRoutableReturn")
            .field("node", &self.node)
            .field("view_graph", &self.view_graph)
            .field("process_blueprint", &"Fn")
            .field("graph_lock", &self.graph_lock)
            .finish()
    }
}
/// If you are providing an initial blueprint, it will be assumed that the ID is a temp_id existing in the blueprint
pub fn use_routable<T: GraphTraits, E: GraphTraits>(
    id: Uid,
    initial_blueprint: Option<BuildBlueprint<T, E>>,
) -> Result<UseRoutableReturn<T, E>, GraphError> {
    let view_graph = use_context::<Rc<RefCell<ViewGraph<T, E>>>>().expect("Context should exist");
    let graph_lock = use_context::<Rc<GraphLock>>().expect("Context should exist");
    let graph_lock_clone = graph_lock.clone();
    let graph_settings = use_context::<Rc<GraphSettings<E>>>().expect("Context should exist");
    let view_graph_clone = view_graph.clone();
    let view_graph_clone_2 = view_graph.clone();

    let process_blueprint = Rc::new(
        move |blueprint: BuildBlueprint<T, E>,
              entry_point_temp_id: Option<TempId>|
              -> Result<(), GraphError> {
            graph_lock.lock();
            let finalized_blueprint = blueprint
                .finalize(
                    &view_graph.clone().borrow(),
                    graph_settings.render_edge_types.clone(),
                    entry_point_temp_id,
                )
                .unwrap();
            // log!("finalized blueprint: {:?}", finalized_blueprint);
            log_finalize_results(&finalized_blueprint);
            {
                let mut graph = view_graph.borrow_mut();

                graph.add_nodes(finalized_blueprint.new_nodes);
                graph.delete_nodes(finalized_blueprint.delete_nodes)?;
            }

            view_graph
                .borrow()
                .update_nodes(finalized_blueprint.update_nodes)?;
            graph_lock.unlock();
            Ok(())
        },
    );

    let mut created_id = None;
    if let Some(initial_blueprint) = initial_blueprint {
        created_id = initial_blueprint.temp_id_map.borrow().get(&id).copied();
        process_blueprint(initial_blueprint, Some(id))?;
    }

    let final_id = created_id.unwrap_or(id);

    let read_reactive_node = view_graph_clone
        .borrow()
        .nodes
        .get(&final_id)
        .ok_or_else(|| {
            GraphError::Blueprint(format!(
                "Use Routable: Failed to find node, ID: {:?}",
                final_id
            ))
        })?
        .0
        .clone();

    // log!("graph len: {}", view_graph_clone.borrow().nodes.len());
    Ok(UseRoutableReturn {
        node: read_reactive_node,
        view_graph: view_graph_clone_2,
        process_blueprint,
        graph_lock: graph_lock_clone,
    })
}
