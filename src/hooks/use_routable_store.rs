use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use im::Vector;
use leptos::*;

use crate::prelude::{
    new_node::TempId, utils::log_finalize_results, view_graph::ViewGraph,
    AllowedRenderEdgeSpecifier, BuildBlueprint, GraphError, GraphTraits, Uid,
};

use super::UseRoutableReturn;

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

pub struct GraphSettings<E: GraphTraits, A: GraphTraits> {
    pub render_edge_types: Option<Vector<AllowedRenderEdgeSpecifier<E>>>,
    pub action_types: PhantomData<A>,
}

pub fn use_routable_store<T: GraphTraits, E: GraphTraits, A: GraphTraits>(
    render_edge_types: Option<impl IntoIterator<Item = AllowedRenderEdgeSpecifier<E>>>,
) {
    let view_graph = Rc::new(RefCell::new(ViewGraph::<T, E, A>::new()));
    let view_graph_clone = view_graph.clone();
    let (is_locked, set_is_locked) = create_signal(false);
    let graph_lock = Rc::new(GraphLock {
        is_locked,
        set_is_locked,
    });
    let graph_lock_clone = graph_lock.clone();

    let graph_settings = Rc::new(GraphSettings {
        render_edge_types: render_edge_types.map(|i| i.into_iter().collect()),
        action_types: PhantomData::<A>,
    });

    let underlying_process_blueprint = Rc::new(
        move |blueprint: BuildBlueprint<T, E>,
              action_data: A,
              entry_point_temp_id: Option<TempId>|
              -> Result<(), GraphError> {
            graph_lock_clone.lock();
            let finalized_blueprint = blueprint
                .finalize(
                    &view_graph_clone.clone().borrow(),
                    graph_settings.render_edge_types.clone(),
                    entry_point_temp_id,
                )
                .unwrap();
            log_finalize_results(&finalized_blueprint);
            {
                let mut graph = view_graph_clone.borrow_mut();

                graph.add_nodes(finalized_blueprint.new_nodes, action_data.clone());
                graph.delete_nodes(finalized_blueprint.delete_nodes)?;
            }

            view_graph_clone
                .borrow()
                .update_nodes(finalized_blueprint.update_nodes, action_data)?;
            graph_lock_clone.unlock();
            Ok(())
        },
    );
    let underlying_process_blueprint_clone = underlying_process_blueprint.clone();

    let initiate_graph = Rc::new(
        move |blueprint: BuildBlueprint<T, E>,
              action_data: A,
              entry_point_temp_id: TempId|
              -> Result<Uid, GraphError> {
            let final_id = blueprint
                .temp_id_map
                .borrow()
                .get(&entry_point_temp_id)
                .copied();
            underlying_process_blueprint_clone(blueprint, action_data, Some(entry_point_temp_id))?;
            if let Some(final_id) = final_id {
                Ok(final_id)
            } else {
                Err(GraphError::Blueprint(format!(
                    "Use Routable: Failed to find given temp entry point, ID: {:?}",
                    entry_point_temp_id
                )))?
            }
        },
    );
    let process_blueprint = Rc::new(
        move |blueprint: BuildBlueprint<T, E>, action_data: A| -> Result<(), GraphError> {
            underlying_process_blueprint(blueprint, action_data, None)
        },
    );

    let get_node = Rc::new(move |id: Uid| {
        let graph = view_graph.borrow();
        let node = graph
            .nodes
            .get(&id)
            .ok_or_else(|| {
                GraphError::Blueprint(format!("Use Routable: Failed to find node, ID: {:?}", id))
            })?
            .0
            .clone();
        Ok(node)
    });

    provide_context(Rc::new(UseRoutableReturn {
        get_node_closure: get_node,
        process_blueprint_closure: process_blueprint,
        initiate_graph_closure: initiate_graph,
        graph_lock,
    }));
}
