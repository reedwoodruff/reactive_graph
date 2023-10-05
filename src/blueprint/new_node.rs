use im::HashSet;
use uuid::Uuid;

use crate::{EdgeDescriptor, GraphError, GraphTraits, Uid};

pub type TempId = Uid;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct NewNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub temp_id: Option<TempId>,
    pub data: T,
    pub add_labels: HashSet<String>,
    pub add_edges: HashSet<EdgeDescriptor<E>>,
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

    pub fn merge(&mut self, other: Self) -> Result<(), GraphError> {
        if self.id != other.id {
            return Err(GraphError::Blueprint(
                format!(
                    "cannot merge nodes with different ids\nID1: {:?}\nID2: {:?}",
                    self.id, other.id
                ),
            ));
        }
        if self.temp_id.is_none() {
            self.temp_id = other.temp_id;
        }
        if self.data == T::default() {
            self.data = other.data;
        }
        self.add_labels.extend(other.add_labels);
        self.add_edges.extend(other.add_edges);

        Ok(())
    }

    pub fn set_id(mut self, id: Uid) -> Self {
        self.id = id;
        self
    }
}
