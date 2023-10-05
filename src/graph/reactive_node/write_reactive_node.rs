use im::HashSet;

use crate::{blueprint::update_node::UpdateNode, EdgeDir};

use super::super::{
    common::{EdgeDescriptor, Uid},
    GraphTraits,
};
use im::hashmap::HashMap;
use leptos::*;

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct WriteReactiveNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub data: WriteSignal<T>,
    pub labels: WriteSignal<HashSet<String>>,
    pub incoming_edges: WriteSignal<HashMap<E, HashSet<EdgeDescriptor<E>>>>,
    pub outgoing_edges: WriteSignal<HashMap<E, HashSet<EdgeDescriptor<E>>>>,
}

impl<T: GraphTraits, E: GraphTraits> WriteReactiveNode<T, E> {
    pub fn update(&mut self, node: UpdateNode<T, E>) {
        if let Some(data) = node.replacement_data {
            self.data.set(data);
        }
        if let Some(labels) = node.add_labels {
            self.add_labels(labels);
        }
        if let Some(labels) = node.remove_labels {
            self.remove_labels(labels);
        }
        if let Some(edges) = node.add_edges {
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

        if let Some(edges) = node.remove_edges {
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
    }

    fn add_labels(&self, labels: HashSet<String>) {
        self.labels.update(|prev| {
            prev.extend(labels);
        });
    }

    fn remove_labels(&mut self, labels: HashSet<String>) {
        self.labels.update(|prev| {
            let new_list = prev.clone().relative_complement(labels);
            prev.clear();
            prev.extend(new_list);
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
                    .remove(&edge);
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
                    .insert(edge.clone());
            }
        });
    }
}
