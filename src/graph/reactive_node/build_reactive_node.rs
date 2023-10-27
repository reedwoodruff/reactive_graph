use std::rc::Rc;

use im::hashmap::HashMap;

use im::{HashSet, Vector};
// use leptos_reactive::{create_signal, Signal};
use leptos::*;

use crate::blueprint::new_node::NewNode;
use crate::prelude::*;

use super::last_action::{ActionData, LastAction};
use super::read_reactive_node::ReadReactiveNode;
use super::write_reactive_node::WriteReactiveNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuildReactiveNode<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    id: Option<Uid>,
    data: Option<T>,
    labels: Option<Vector<String>>,
    incoming_edges: Option<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    outgoing_edges: Option<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    last_action: Option<LastAction<T, E, A>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Default for BuildReactiveNode<T, E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> BuildReactiveNode<T, E, A> {
    // Creates a new ConstructNode with all fields set to None
    pub fn new() -> Self {
        Self {
            id: None,
            data: None,
            labels: None,
            incoming_edges: None,
            outgoing_edges: None,
            last_action: None,
        }
    }

    // Sets the id field and returns self for chaining
    pub fn id(&self, id: Uid) -> Self {
        Self {
            id: Some(id),
            ..self.clone()
        }
    }

    // Sets the data field and returns self for chaining
    pub fn data(&self, data: T) -> Self {
        Self {
            data: Some(data),
            ..self.clone()
        }
    }

    // Sets the labels field and returns self for chaining
    pub fn add_labels<I>(&self, labels: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut updated_labels = self.labels.clone().unwrap_or_default();
        updated_labels.extend(labels);
        Self {
            labels: Some(updated_labels),
            ..self.clone()
        }
    }

    // Sets the incoming_edges field and returns self for chaining
    pub fn add_incoming_edges<I>(&self, incoming_edges: I) -> Self
    where
        I: IntoIterator<Item = (E, Vector<EdgeDescriptor<E>>)>,
    {
        let mut updated_incoming_edges = self.incoming_edges.clone().unwrap_or_default();
        updated_incoming_edges.extend(incoming_edges);
        Self {
            incoming_edges: Some(updated_incoming_edges),
            ..self.clone()
        }
    }

    // Sets the outgoing_edges field and returns self for chaining
    pub fn add_outgoing_edges<I>(&self, outgoing_edges: I) -> Self
    where
        I: IntoIterator<Item = (E, Vector<EdgeDescriptor<E>>)>,
    {
        let mut updated_outgoing_edges = self.outgoing_edges.clone().unwrap_or_default();
        updated_outgoing_edges.extend(outgoing_edges);
        Self {
            outgoing_edges: Some(updated_outgoing_edges),
            ..self.clone()
        }
    }

    pub fn add_last_action(&self, last_action: LastAction<T, E, A>) -> Self {
        Self {
            last_action: Some(last_action),
            ..self.clone()
        }
    }

    pub fn map_edges_from_bp(&self, edges: &HashSet<EdgeDescriptor<E>>) -> Self {
        let mut new_outgoing_edges = self.outgoing_edges.clone().unwrap_or_default();
        let mut new_incoming_edges = self.incoming_edges.clone().unwrap_or_default();

        for edge in edges {
            let map_to_edit = match edge.dir {
                EdgeDir::Emit => &mut new_outgoing_edges,
                EdgeDir::Recv => &mut new_incoming_edges,
            };
            let edge_list = map_to_edit.entry(edge.edge_type.clone()).or_default();
            if !edge_list.contains(edge) {
                edge_list.push_back(edge.clone());
            }
        }

        Self {
            incoming_edges: Some(new_incoming_edges),
            outgoing_edges: Some(new_outgoing_edges),
            ..self.clone()
        }
    }

    pub fn ingest_from_blueprint(&self, bp: NewNode<T, E>, action_data: Rc<ActionData<A>>) -> Self {
        self.data(bp.data)
            .id(bp.id)
            .map_edges_from_bp(&bp.add_edges)
            .add_labels(bp.add_labels)
            .add_last_action(LastAction::<T, E, A> {
                action_data,
                update_info: None,
            })
    }

    // Constructs the final Node object
    pub fn build(&self) -> (ReadReactiveNode<T, E, A>, WriteReactiveNode<T, E, A>) {
        let (read_data, write_data) =
            create_signal::<T>(self.data.clone().expect("Data must be set"));
        let (read_labels, write_labels) =
            create_signal::<Vector<String>>(self.labels.clone().unwrap_or_default());
        let (read_incoming_edges, write_incoming_edges) =
            create_signal::<HashMap<E, Vector<EdgeDescriptor<E>>>>(
                self.incoming_edges.clone().unwrap_or_default(),
            );
        let (read_outgoing_edges, write_outgoing_edges) =
            create_signal::<HashMap<E, Vector<EdgeDescriptor<E>>>>(
                self.outgoing_edges.clone().unwrap_or_default(),
            );

        let (read_last_action, write_last_action) = create_signal::<LastAction<T, E, A>>(
            self.last_action
                .clone()
                .expect("Last action must be set")
                .clone(),
        );
        let read_node = ReadReactiveNode {
            id: self.id.expect("Node ID should be provided"),
            data: read_data,
            labels: read_labels,
            incoming_edges: read_incoming_edges,
            outgoing_edges: read_outgoing_edges,
            last_action: read_last_action,
        };
        let write_node = WriteReactiveNode {
            id: self.id.expect("Node ID should be provided"), // Default to new ID if not set
            data: write_data,
            labels: write_labels,
            incoming_edges: write_incoming_edges,
            outgoing_edges: write_outgoing_edges,
            last_action: write_last_action,
        };

        (read_node, write_node)
    }
}
