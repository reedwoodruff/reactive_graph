use leptos::logging::log;

use crate::prelude::GraphTraits;

use super::FinalizedBlueprint;

pub fn log_finalize_results<T: GraphTraits, E: GraphTraits>(
    build_blueprint: &FinalizedBlueprint<T, E>,
) {
    for node in build_blueprint.new_nodes.values() {
        log!("+ New Node {:?}", node.id);
        if node.temp_id.is_some() {
            log!("  + Temp ID: {:?}", node.temp_id);
        }
        if node.data != T::default() {
            log!("  + Data: {:?}", node.data);
        }
        for edge in node.add_edges.iter() {
            log!("  + Edge: {:?}", edge);
        }
    }
    for node in build_blueprint.update_nodes.values() {
        log!("~ Update Node {:?}", node.id);
        if node.replacement_data.is_some() {
            log!("  ~ Data: {:?}", node.replacement_data);
        }
        for edge in node.add_edges.iter() {
            log!("  + Edge: {:?}", edge);
        }
        for edge in node.remove_edges.iter() {
            log!("  - Edge: {:?}", edge);
        }
        for label in node.add_labels.iter() {
            log!("  + Label: {:?}", label);
        }
        for label in node.remove_labels.iter() {
            log!("  - Label: {:?}", label);
        }
    }
    for node in build_blueprint.delete_nodes.iter() {
        log!("- Delete Node {:?}", node);
    }
}
