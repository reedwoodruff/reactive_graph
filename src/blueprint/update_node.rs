use im::HashSet;

use crate::{EdgeDescriptor, GraphError, GraphTraits, Uid};

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

    pub fn merge(&mut self, other: Self) -> Result<(), GraphError> {
        if self.id != other.id {
            return Err(GraphError::Blueprint(
                format!(
                    "cannot merge nodes with different ids\nID1: {:?}\nID2: {:?}",
                    self.id, other.id
                ),
            ));
        }
        if other.replacement_data.is_some() {
            self.replacement_data = other.replacement_data;
        }
        if let Some(other_labels) = other.add_labels {
            self.add_labels = Some(
                self.add_labels
                    .clone()
                    .unwrap_or_default()
                    .union(other_labels),
            );
        }

        if let Some(other_labels) = other.remove_labels {
            self.remove_labels = Some(
                self.remove_labels
                    .clone()
                    .unwrap_or_default()
                    .union(other_labels),
            );
        }

        if let Some(other_edges) = other.add_edges {
            self.add_edges = Some(
                self.add_edges
                    .clone()
                    .unwrap_or_default()
                    .union(other_edges),
            );
        }

        if let Some(other_edges) = other.remove_edges {
            self.remove_edges = Some(
                self.remove_edges
                    .clone()
                    .unwrap_or_default()
                    .union(other_edges),
            );
        }

        Ok(())
    }
}
