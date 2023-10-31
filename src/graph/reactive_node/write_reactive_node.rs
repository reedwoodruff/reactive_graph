use std::rc::Rc;

use im::{HashSet, Vector};
// use leptos_reactive::{SignalSet, SignalUpdate, WriteSignal};
use leptos::*;

use crate::prelude::finalized_update_node::FinalizedUpdateNode;
use crate::prelude::*;

use super::last_action::ActionData;
use super::last_action::LastAction;
use im::hashmap::HashMap;

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct WriteReactiveNode<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub id: Uid,
    pub data: WriteSignal<T>,
    pub labels: WriteSignal<Vector<String>>,
    pub incoming_edges: WriteSignal<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    pub outgoing_edges: WriteSignal<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    pub last_action: WriteSignal<LastAction<T, E, A>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> WriteReactiveNode<T, E, A> {
    pub fn update(
        &mut self,
        update_node: FinalizedUpdateNode<T, E>,
        action_data: Rc<ActionData<A>>,
    ) {
        let new_last_action = LastAction {
            action_data,
            update_info: Some(update_node.clone()),
        };
        self.last_action.set(new_last_action);

        // log!("Updating signal, node: {:?}", node.id);
        if let Some(data) = update_node.replacement_data {
            self.data.set(data.new_data);
        }
        if let Some(labels) = update_node.add_labels {
            self.add_labels(labels);
        }
        if let Some(labels) = update_node.remove_labels {
            self.remove_labels(labels);
        }
        if let Some(edges) = update_node.remove_edges {
            let mut incoming_edges = edges.clone();
            incoming_edges.retain(|edge| edge.dir == EdgeDir::Recv);
            let mut outgoing_edges = edges.clone();
            outgoing_edges.retain(|edge| edge.dir == EdgeDir::Emit);
            if !incoming_edges.is_empty() {
                self.remove_edges(incoming_edges, EdgeDir::Recv);
            }
            if !outgoing_edges.is_empty() {
                self.remove_edges(outgoing_edges, EdgeDir::Emit);
            }
        }

        if let Some(edges) = update_node.add_edges {
            let mut incoming_edges = edges.clone();
            incoming_edges.retain(|edge| edge.dir == EdgeDir::Recv);
            let mut outgoing_edges = edges.clone();
            outgoing_edges.retain(|edge| edge.dir == EdgeDir::Emit);
            if !incoming_edges.is_empty() {
                self.add_edges(incoming_edges, EdgeDir::Recv);
            }
            if !outgoing_edges.is_empty() {
                self.add_edges(outgoing_edges, EdgeDir::Emit);
            }
        }
    }

    fn add_labels(&self, labels: HashSet<String>) {
        self.labels.update(|prev| {
            prev.extend(labels);
        });
    }

    fn remove_labels(&mut self, labels: HashSet<String>) {
        self.labels.update(|prev| {
            let new_labels = prev
                .iter()
                .filter(|&x| !labels.contains(x))
                .cloned()
                .collect();
            prev.clear();
            prev.append(new_labels);
        });
    }

    fn remove_edges(&mut self, edges: HashSet<EdgeDescriptor<E>>, direction: EdgeDir) {
        let map_to_edit = match direction {
            EdgeDir::Recv => &mut self.incoming_edges,
            EdgeDir::Emit => &mut self.outgoing_edges,
        };

        map_to_edit.update(|prev| {
            for edge in edges {
                if prev.get(&edge.edge_type).is_none() {
                    panic!("Tried to remove edge that doesn't exist");
                }
                prev.entry(edge.edge_type.clone())
                    .or_default()
                    .retain(|x| x != &edge);
                if prev.get(&edge.edge_type).unwrap().is_empty() {
                    prev.remove(&edge.edge_type);
                }
            }
        });
    }

    fn add_edges(&mut self, edges: HashSet<EdgeDescriptor<E>>, direction: EdgeDir) {
        let map_to_edit = match direction {
            EdgeDir::Recv => &mut self.incoming_edges,
            EdgeDir::Emit => &mut self.outgoing_edges,
        };

        map_to_edit.update(|prev| {
            for edge in edges {
                prev.entry(edge.edge_type.clone())
                    .or_default()
                    .push_back(edge.clone());
            }
        });
    }

    // fn set_last_action(&mut self, action: LastAction<T, E, A>) {
    //     self.last_action.set(action);
    // }
}
