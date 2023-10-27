use im::HashMap;

use crate::prelude::{GraphTraits, Uid};

use super::{
    delete_node::DeleteNode, finalized_update_node::FinalizedUpdateNode, new_node::NewNode,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FinalizedBlueprint<T: GraphTraits, E: GraphTraits> {
    pub new_nodes: HashMap<Uid, NewNode<T, E>>,
    pub update_nodes: HashMap<Uid, FinalizedUpdateNode<T, E>>,
    pub delete_nodes: HashMap<Uid, DeleteNode<T, E>>,
}

impl<T: GraphTraits, E: GraphTraits> FinalizedBlueprint<T, E> {
    pub fn invert_blueprint(&self) -> Self {
        let inverted_update_nodes = self
            .update_nodes
            .iter()
            .map(|(id, node)| (*id, node.history_invert()))
            .collect();
        let inverted_delete_nodes = self
            .delete_nodes
            .iter()
            .map(|(id, node)| (*id, node.history_invert()))
            .collect();
        let inverted_new_nodes = self
            .new_nodes
            .iter()
            .map(|(id, node)| (*id, node.history_invert()))
            .collect();

        Self {
            new_nodes: inverted_delete_nodes,
            update_nodes: inverted_update_nodes,
            delete_nodes: inverted_new_nodes,
        }
    }
}
