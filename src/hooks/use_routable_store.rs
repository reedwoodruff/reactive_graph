use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use im::Vector;
use leptos::*;

use crate::{
    history::{
        self,
        history_interface::HistoryInterface,
        history_store::{reactive_node::last_action::ActionData, FinalizedBlueprint, HistoryStore},
    },
    prelude::{
        new_node::TempId, view_graph::ViewGraph, AllowedRenderEdgeSpecifier, BuildBlueprint,
        GraphError, GraphTraits, Uid,
    },
};

use super::{ProcessBlueprintReturn, UseRoutableReturn};

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
    let view_graph_clone2 = view_graph.clone();
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

    let apply_finalized_blueprint = Rc::new(
        move |finalized_blueprint: FinalizedBlueprint<T, E>,
              primary_action_data: A,
              secondary_action_data: Option<Rc<A>>|
              -> Result<(), GraphError> {
            // log_finalize_results(&finalized_blueprint);
            graph_lock_clone.lock();
            let mut action_data = ActionData::<A>::new(primary_action_data);
            if let Some(secondary_action_data) = secondary_action_data {
                action_data = action_data.set_secondary_action(secondary_action_data);
            }
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
    let apply_finalized_blueprint_clone = apply_finalized_blueprint.clone();

    let history_interface = Rc::new(HistoryInterface::<T, E, A>::new(
        HistoryStore::<T, E, A>::new(),
        apply_finalized_blueprint_clone,
    ));
    let history_interface_clone1 = history_interface.clone();

    let underlying_process_blueprint = Rc::new(
        move |blueprint: BuildBlueprint<T, E, A>,
              action_data: A,
              entry_point_temp_id: Option<TempId>|
              -> Result<(), GraphError> {
            let finalized_blueprint = blueprint
                .finalize(
                    &view_graph_clone2.clone().borrow(),
                    graph_settings.render_edge_types.clone(),
                    entry_point_temp_id,
                )
                .unwrap();
            history_interface_clone1.push(history::history_item::HistoryItem {
                blueprint: finalized_blueprint.clone(),
                action_data: Rc::new(action_data.clone()),
            });
            apply_finalized_blueprint(finalized_blueprint, action_data, None)
        },
    );
    let underlying_process_blueprint_clone = underlying_process_blueprint.clone();

    let initiate_graph = Rc::new(
        move |blueprint: BuildBlueprint<T, E, A>,
              action_data: A,
              entry_point_temp_id: TempId|
              -> ProcessBlueprintReturn {
            // let final_id = blueprint
            //     .temp_id_map
            //     .borrow()
            //     .get(&entry_point_temp_id)
            //     .copied();
            let temp_id_map = blueprint.temp_id_map.borrow().clone();
            underlying_process_blueprint_clone(blueprint, action_data, Some(entry_point_temp_id))?;
            // if let Some(final_id) = final_id {
            //     Ok(final_id)
            // } else {
            //     Err(GraphError::Blueprint(format!(
            //         "Use Routable: Failed to find given temp entry point, ID: {:?}",
            //         entry_point_temp_id
            //     )))?
            // }
            Ok(temp_id_map)
        },
    );
    let process_blueprint = Rc::new(
        move |blueprint: BuildBlueprint<T, E, A>, action_data: A| -> ProcessBlueprintReturn {
            let temp_id_map = blueprint.temp_id_map.borrow().clone();
            underlying_process_blueprint(blueprint, action_data, None)?;
            Ok(temp_id_map)
        },
    );

    let get_node = Rc::new(move |id: &Uid| {
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
        history: history_interface,
    }));
}
