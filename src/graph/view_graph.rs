use leptos::*;
use std::{cell::RefCell, rc::Rc};

use im::hashmap::HashMap;

use crate::{
    blueprint::{new_node::NewNode, update_node::UpdateNode},
    // blueprint::{
    //     existing_blueprint_node::flat_existing_blueprint_node::FlatExistingBpNode,
    //     new_blueprint_node::flat_new_blueprint_node::FlatNewBpNode,
    // },
    reactive_node::{
        build_reactive_node::BuildReactiveNode, read_reactive_node::ReadReactiveNode,
        write_reactive_node::WriteReactiveNode,
    },
    GraphTraits,
    Uid,
};
use im::Vector;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ViewGraph<T: GraphTraits, E: GraphTraits> {
    pub nodes: HashMap<Uid, (Rc<ReadReactiveNode<T, E>>, RefCell<WriteReactiveNode<T, E>>)>,
    pub label_map: HashMap<String, Vector<Uid>>,
}

impl<T: GraphTraits, E: GraphTraits> ViewGraph<T, E> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            label_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: NewNode<T, E>) {
        let id = node.id;
        let (read_node, write_node) = BuildReactiveNode::new().ingest_from_blueprint(node).build();
        for label in read_node.labels.get().iter() {
            let mut nodes_with_label = self.label_map.get(label).unwrap_or(&Vector::new()).clone();
            nodes_with_label.push_back(id);
            self.label_map.insert(label.clone(), nodes_with_label);
        }
        self.nodes
            .insert(id, (Rc::new(read_node), RefCell::new(write_node)));
    }

    pub fn update_node(&self, node: UpdateNode<T, E>) -> Result<(), String> {
        self.nodes
            .get(&node.id)
            .ok_or_else(|| format!("Update Node: Failed to find node, ID: {:?}", node.id))?
            .1
            .borrow_mut()
            .update(node);
        Ok(())
    }

    pub fn delete_node(&mut self, node_id: Uid) -> Result<(), String> {
        let node = self
            .nodes
            .get(&node_id)
            .ok_or_else(|| format!("Delete Node: Failed to find node, ID: {:?}", node_id))?;
        let node = &node.0;
        for label in node.labels.get().iter() {
            let mut nodes_with_label = self
                .label_map
                .get(label)
                .ok_or_else(|| format!("Delete Node: Failed to find label, Label: {:?}", label))?
                .clone();
            nodes_with_label.retain(|item| *item != node_id);
            self.label_map.insert(label.clone(), nodes_with_label);
        }
        self.nodes.remove(&node_id);
        Ok(())
    }
}
