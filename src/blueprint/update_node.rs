use im::HashSet;
use leptos::error::Error;

use crate::{EdgeDescriptor, EdgeDir, GraphError, GraphTraits, Uid};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct UpdateNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub replacement_data: Option<T>,
    pub add_labels: Option<HashSet<String>>,
    pub remove_labels: Option<HashSet<String>>,
    pub add_edges: Option<HashSet<EdgeDescriptor<E>>>,
    pub remove_edges: Option<HashSet<EdgeDescriptor<E>>>,
}

impl<T: GraphTraits, E: GraphTraits> UpdateNode<T, E> {
    pub fn new(id: Uid) -> Self {
        Self {
            id,
            replacement_data: None,
            add_labels: None,
            remove_labels: None,
            add_edges: None,
            remove_edges: None,
        }
    }

    pub fn merge(&self, other: Self) -> Result<Self, GraphError> {
        if self.id != other.id {
            return Err(GraphError::Blueprint(format!(
                "cannot merge nodes with different ids\nID1: {:?}\nID2: {:?}",
                self.id, other.id
            )));
        }
        Ok(Self {
            id: self.id,
            replacement_data: if self.replacement_data.is_none() {
                other.replacement_data
            } else {
                self.replacement_data.clone()
            },
            add_labels: if let Some(other_labels) = other.add_labels {
                Some(
                    self.add_labels
                        .clone()
                        .unwrap_or_default()
                        .union(other_labels),
                )
            } else {
                self.add_labels.clone()
            },
            remove_labels: if let Some(other_labels) = other.remove_labels {
                Some(
                    self.remove_labels
                        .clone()
                        .unwrap_or_default()
                        .union(other_labels),
                )
            } else {
                self.remove_labels.clone()
            },
            add_edges: if let Some(other_edges) = other.add_edges {
                Some(
                    self.add_edges
                        .clone()
                        .unwrap_or_default()
                        .union(other_edges),
                )
            } else {
                self.add_edges.clone()
            },
            remove_edges: if let Some(other_edges) = other.remove_edges {
                Some(
                    self.remove_edges
                        .clone()
                        .unwrap_or_default()
                        .union(other_edges),
                )
            } else {
                self.remove_edges.clone()
            },
        })
    }

    pub fn update_edge_render_info(
        &self,
        edge: &EdgeDescriptor<E>,
        new_render_info: Option<EdgeDir>,
    ) -> Result<Self, GraphError> {
        let new_edges = self.add_edges.clone();
        if new_edges.is_none() {
            return Err(GraphError::Blueprint(
                "cannot update edge render info on node with no edges".to_string(),
            ));
        }
        let mut new_edges = new_edges.unwrap();

        new_edges.remove(edge);
        new_edges.insert(EdgeDescriptor {
            render_info: new_render_info,
            ..edge.clone()
        });
        Ok(Self {
            add_edges: Some(new_edges),
            ..self.clone()
        })
    }
}
