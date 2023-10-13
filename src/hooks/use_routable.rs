use std::{cell::RefCell, rc::Rc};

use crate::prelude::{
    new_node::TempId, reactive_node::read_reactive_node::ReadReactiveNode, view_graph::ViewGraph, *,
};
use im::Vector;
use leptos::*;

/// If you are providing an initial blueprint, it will be assumed that the ID is a temp_id existing in the blueprint
pub fn use_routable<T: GraphTraits, E: GraphTraits>(
    id: Uid,
    render_edge_types: Option<Vector<AllowedRenderEdgeSpecifier<E>>>,
    initial_blueprint: Option<BuildBlueprint<T, E>>,
) -> Result<
    (
        Rc<ReadReactiveNode<T, E>>,
        Rc<RefCell<ViewGraph<T, E>>>,
        impl Fn(BuildBlueprint<T, E>, Option<TempId>) -> Result<(), GraphError>,
    ),
    GraphError,
> {
    let view_graph = use_context::<Rc<RefCell<ViewGraph<T, E>>>>().expect("Context should exist");
    let graph_lock = use_context::<Rc<GraphLock>>().expect("Context should exist");
    let view_graph_clone = view_graph.clone();
    let view_graph_clone_2 = view_graph.clone();

    let process_blueprint = move |blueprint: BuildBlueprint<T, E>,
                                  entry_point_temp_id: Option<TempId>|
          -> Result<(), GraphError> {
        graph_lock.lock();
        let finalized_blueprint = blueprint
            .finalize(
                &view_graph.clone().borrow(),
                render_edge_types.clone(),
                entry_point_temp_id,
            )
            .unwrap();
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
    };

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
            GraphError::Blueprint(format!("Use Routable: Failed to find node, ID: {:?}", id))
        })?
        .0
        .clone();

    // log!("graph len: {}", view_graph_clone.borrow().nodes.len());
    Ok((read_reactive_node, view_graph_clone_2, process_blueprint))
}
