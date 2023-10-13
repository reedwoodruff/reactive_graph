use im::HashSet;


use crate::prelude::*;

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
        })
    }

    pub fn update_edge_render_info(
        &self,
        edge: &EdgeDescriptor<E>,
        new_render_info: Option<EdgeDir>,
    ) -> Self {
        let new_edges = self.add_edges.clone();
        let mut remove_edges = self.remove_edges.clone();

        // If the edge in question is not in the new edges, then it must be an existing edge
        // As such we need to delete the existing edge in order to create the new edge with the specified render information
        if new_edges.is_none() {
            let mut new_remove_edges = remove_edges.unwrap_or_default();
            new_remove_edges.insert(edge.clone());
            remove_edges = Some(new_remove_edges);
        }
        let mut new_edges = new_edges.unwrap_or_default();

        new_edges.remove(edge);
        new_edges.insert(EdgeDescriptor {
            render_info: new_render_info,
            ..edge.clone()
        });
        Self {
            add_edges: Some(new_edges),
            remove_edges,
            ..self.clone()
        }
    }
}
