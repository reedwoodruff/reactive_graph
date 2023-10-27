use im::HashSet;

use crate::prelude::{EdgeDescriptor, GraphTraits, Uid};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct UpdateNodeReplacementData<T: GraphTraits> {
    pub new_data: T,
    pub prev_data: T,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct FinalizedUpdateNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub replacement_data: Option<UpdateNodeReplacementData<T>>,
    pub add_labels: Option<HashSet<String>>,
    pub remove_labels: Option<HashSet<String>>,
    pub add_edges: Option<HashSet<EdgeDescriptor<E>>>,
    pub remove_edges: Option<HashSet<EdgeDescriptor<E>>>,
}

impl<T: GraphTraits, E: GraphTraits> FinalizedUpdateNode<T, E> {
    pub fn history_invert(&self) -> Self {
        Self {
            id: self.id,
            replacement_data: self
                .replacement_data
                .clone()
                .map(|data| UpdateNodeReplacementData {
                    new_data: data.prev_data,
                    prev_data: data.new_data,
                }),
            add_labels: self.remove_labels.clone(),
            remove_labels: self.add_labels.clone(),
            add_edges: self.remove_edges.clone(),
            remove_edges: self.add_edges.clone(),
        }
    }
}
