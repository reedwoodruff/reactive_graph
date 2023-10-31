use im::HashSet;
use uuid::Uuid;

use crate::prelude::*;

use super::delete_node::DeleteNode;

pub type TempId = Uid;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NewNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub temp_id: Option<TempId>,
    pub data: T,
    pub add_labels: HashSet<String>,
    pub add_edges: HashSet<EdgeDescriptor<E>>,
}

impl<T: GraphTraits, E: GraphTraits> Default for NewNode<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: GraphTraits, E: GraphTraits> NewNode<T, E> {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_u128_le(),
            temp_id: None,
            data: T::default(),
            add_labels: HashSet::new(),
            add_edges: HashSet::new(),
        }
    }

    pub fn merge_additive(&self, other: Self) -> Result<Self, GraphError> {
        if self.id != other.id {
            return Err(GraphError::Blueprint(format!(
                "cannot merge nodes with different ids\nID1: {:?}\nID2: {:?}",
                self.id, other.id
            )));
        }
        Ok(Self {
            id: self.id,
            temp_id: if self.temp_id.is_none() {
                other.temp_id
            } else {
                self.temp_id
            },
            data: if self.data == T::default() {
                other.data
            } else {
                self.data.clone()
            },
            add_labels: self.add_labels.clone().union(other.add_labels),
            add_edges: self.add_edges.clone().union(other.add_edges),
        })
    }

    pub fn set_id(&self, id: Uid) -> Self {
        Self { id, ..self.clone() }
    }

    pub fn find_edges<A: GraphTraits>(
        &self,
        edge_finder: &EdgeFinder<T, E, A>,
    ) -> HashSet<EdgeDescriptor<E>> {
        let mut edges = HashSet::new();
        for edge in self.add_edges.iter() {
            if (edge_finder.match_all.is_none()
                || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap()))
                && !edges.is_empty()
            {
                break;
            }

            if edge_finder.matches(edge) {
                edges.insert(edge.clone());
            }
        }
        edges
    }

    pub fn update_edge_render_info(
        &self,
        edge: &EdgeDescriptor<E>,
        new_render_info: Option<EdgeDir>,
    ) -> Self {
        let mut new_edges = self.add_edges.clone();
        new_edges.remove(edge);
        new_edges.insert(EdgeDescriptor {
            render_info: new_render_info,
            ..edge.clone()
        });

        Self {
            add_edges: new_edges,
            ..self.clone()
        }
    }

    pub fn get_render_edge<A: GraphTraits>(&self) -> Option<EdgeDescriptor<E>> {
        let render_edge =
            self.find_edges(&EdgeFinder::<T, E, A>::new().render_info(Some(EdgeDir::Recv)));
        if render_edge.is_empty() {
            None
        } else {
            Some(render_edge.iter().next().unwrap().clone())
        }
    }

    pub fn history_invert(&self) -> DeleteNode<T, E> {
        DeleteNode {
            id: self.id,
            data: self.data.clone(),
            remove_labels: self.add_labels.clone(),
            remove_edges: self.add_edges.clone(),
        }
    }
}
