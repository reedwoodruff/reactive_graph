use im::HashSet;
use leptos::{logging::log, SignalGetUntracked};

use crate::prelude::{
    reactive_node::read_reactive_node::ReadReactiveNode, EdgeDescriptor, GraphTraits, Uid,
};

use super::new_node::NewNode;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct DeleteNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub remove_edges: HashSet<EdgeDescriptor<E>>,
    pub remove_labels: HashSet<String>,
    pub data: T,
}

impl<T: GraphTraits, E: GraphTraits> DeleteNode<T, E> {
    pub fn new(
        id: Uid,
        edges: HashSet<EdgeDescriptor<E>>,
        labels: HashSet<String>,
        data: T,
    ) -> Self {
        Self {
            id,
            remove_edges: edges,
            remove_labels: labels,
            data,
        }
    }

    pub fn from_read_reactive_node<A: GraphTraits>(
        reactive_node: &ReadReactiveNode<T, E, A>,
    ) -> Self {
        log!(
            "reactive_node outgoing_edges: {:?}",
            reactive_node.outgoing_edges.get_untracked()
        );
        log!(
            "reactive_node incoming_edges: {:?}",
            reactive_node.outgoing_edges.get_untracked()
        );
        log!(
            "reactive_node to_hashset: {:?}",
            reactive_node.convert_all_edges_to_hashset()
        );

        Self {
            id: reactive_node.id,
            remove_edges: reactive_node.convert_all_edges_to_hashset(),
            remove_labels: reactive_node
                .labels
                .get_untracked()
                .iter()
                .cloned()
                .collect(),
            data: reactive_node.data.get_untracked().clone(),
        }
    }

    pub fn history_invert(&self) -> NewNode<T, E> {
        NewNode {
            id: self.id,
            temp_id: None,
            add_edges: self.remove_edges.clone(),
            add_labels: self.remove_labels.clone(),
            data: self.data.clone(),
        }
    }
}
