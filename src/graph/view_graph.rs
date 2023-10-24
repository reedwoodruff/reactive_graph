// use leptos_reactive::SignalGet;
use leptos::*;
use std::{cell::RefCell, rc::Rc};

use im::{hashmap::HashMap, HashSet};

use crate::prelude::{new_node::NewNode, update_node::UpdateNode, *};
use im::Vector;

use super::reactive_node::{
    build_reactive_node::BuildReactiveNode, read_reactive_node::ReadReactiveNode,
    write_reactive_node::WriteReactiveNode,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewGraph<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub nodes: HashMap<
        Uid,
        (
            Rc<ReadReactiveNode<T, E, A>>,
            RefCell<WriteReactiveNode<T, E, A>>,
        ),
    >,
    pub label_map: HashMap<String, Vector<Uid>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Default for ViewGraph<T, E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> ViewGraph<T, E, A> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            label_map: HashMap::new(),
        }
    }
    pub fn add_nodes(&mut self, nodes: HashMap<Uid, NewNode<T, E>>, action_data: A) {
        for (_id, node) in nodes {
            self.add_node(node, Rc::new(action_data.clone()));
        }
    }
    pub fn delete_nodes(&mut self, nodes: HashSet<Uid>) -> Result<(), GraphError> {
        for node in nodes {
            self.delete_node(node)?;
        }
        Ok(())
    }
    pub fn update_nodes(
        &self,
        update_nodes: HashMap<Uid, UpdateNode<T, E>>,
        action_data: A,
    ) -> Result<(), GraphError> {
        let action_data = Rc::new(action_data);
        for (_id, node) in update_nodes {
            self.update_node(node, action_data.clone())?;
        }
        Ok(())
    }

    fn add_node(&mut self, add_node: NewNode<T, E>, action_data: Rc<A>) {
        let id = add_node.id;
        let (read_node, write_node) = BuildReactiveNode::new()
            .ingest_from_blueprint(add_node, action_data)
            .build();
        for label in read_node.labels.get_untracked().iter() {
            let mut nodes_with_label = self
                .label_map
                .get::<String>(label)
                .unwrap_or(&Vector::new())
                .clone();
            nodes_with_label.push_back(id);
            self.label_map.insert(label.clone(), nodes_with_label);
        }
        self.nodes
            .insert(id, (Rc::new(read_node), RefCell::new(write_node)));
    }

    fn update_node(
        &self,
        update_node: UpdateNode<T, E>,
        action_data: Rc<A>,
    ) -> Result<(), GraphError> {
        let graph_node = self.nodes.get(&update_node.id);
        if let Some(graph_node) = graph_node {
            graph_node.1.borrow_mut().update(update_node, action_data);
        } else {
            return Err(GraphError::Blueprint(format!(
                "Update Node: Failed to find node, ID: {:?}",
                update_node.id
            )));
        }
        Ok(())
    }

    fn delete_node(&mut self, node_id: Uid) -> Result<(), GraphError> {
        let node = self.nodes.get(&node_id).ok_or_else(|| {
            GraphError::Blueprint(format!(
                "Delete Node: Failed to find node, ID: {:?}",
                node_id
            ))
        })?;
        let node = &node.0;
        for label in node.labels.get().iter() {
            let mut nodes_with_label = self
                .label_map
                .get(label)
                .ok_or_else(|| {
                    GraphError::Blueprint(format!(
                        "Delete Node: Failed to find label, Label: {:?}",
                        label
                    ))
                })?
                .clone();
            nodes_with_label.retain(|item| *item != node_id);
            self.label_map.insert(label.clone(), nodes_with_label);
        }
        self.nodes.remove(&node_id);
        Ok(())
    }
}
