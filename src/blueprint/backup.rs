use std::{cell::RefCell, rc::Rc};

use im::{HashMap, HashSet};

use crate::{
    graph::graph::Graph, EdgeDescriptor, EdgeDir, EdgeFinder, GraphError, GraphTraits, Uid,
};

use super::{
    new_node::{NewNode, TempId},
    update_node::UpdateNode,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuildBlueprint<'a, T: GraphTraits, E: GraphTraits> {
    new_nodes: HashMap<Uid, NewNode<T, E>>,
    update_nodes: HashMap<Uid, UpdateNode<T, E>>,
    delete_nodes: HashSet<Uid>,
    graph: &'a Graph<T, E>,
    temp_edges: HashSet<EdgeDescriptor<E>>,
    temp_id_map: HashMap<TempId, Uid>,
}

impl<'a, 'b: 'a, T: GraphTraits, E: GraphTraits> BuildBlueprint<'a, T, E> {
    pub fn new(graph: &'a Graph<T, E>) -> Self {
        Self {
            new_nodes: HashMap::new(),
            update_nodes: HashMap::new(),
            delete_nodes: HashSet::new(),
            graph,
            temp_edges: HashSet::new(),
            temp_id_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: NewNode<T, E>) -> Result<(), GraphError> {
        if let Some(existing_entry) = self.new_nodes.get_mut(&node.id) {
            existing_entry.merge(node)?;
        } else {
            self.new_nodes.insert(node.id.clone(), node);
        }
        Ok(())
        // return Ok(BlueNew {
        //     node: NewNode::new(),
        //     blueprint: self,
        // });
    }

    pub fn start_with_new_node(
        &'b mut self,
        node: NewNode<T, E>,
    ) -> Result<BlueNew<'a, T, E>, GraphError> {
        self.add_node(node)?;
        return Ok(BlueNew {
            node: NewNode::new(),
            blueprint: Rc::new(RefCell::new(self)),
        });
    }

    fn update_node(&mut self, node: UpdateNode<T, E>) -> Result<(), GraphError> {
        if let Some(existing_entry) = self.update_nodes.get_mut(&node.id) {
            existing_entry.merge(node)?;
        } else {
            self.update_nodes.insert(node.id.clone(), node);
        }
        Ok(())
    }

    pub fn start_with_update_node(
        &'b mut self,
        node: UpdateNode<T, E>,
    ) -> Result<BlueUpdate<'b, T, E>, GraphError> {
        self.update_node(node)?;
        return Ok(BlueUpdate {
            node: UpdateNode::new(0),
            blueprint: Rc::new(RefCell::new(self)),
        });
    }

    pub fn delete_node(&mut self, node_id: Uid) {
        self.delete_nodes.insert(node_id);
    }

    fn add_edge(&mut self, edge: EdgeDescriptor<E>) {
        if self.graph.nodes.get(&edge.host).is_some() {
            let update_node = self
                .update_nodes
                .entry(edge.host.clone())
                .or_insert_with(|| UpdateNode::new(edge.host.clone()));

            if update_node.add_edges.is_none() {
                update_node.add_edges = Some(HashSet::new());
            }
            update_node.add_edges.as_mut().unwrap().insert(edge);
            return;
        }

        self.new_nodes
            .entry(edge.host.clone())
            .or_insert_with(|| {
                let new_node = NewNode::new();
                new_node.set_id(edge.host.clone())
            })
            .add_edges
            .insert(edge);
        return;
    }
}

pub struct BlueNew<'a, T: GraphTraits, E: GraphTraits> {
    node: NewNode<T, E>,
    pub blueprint: Rc<RefCell<&'a mut BuildBlueprint<'a, T, E>>>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueNew<'a, T, E> {
    pub fn set_data(mut self, data: T) -> Self {
        self.node.data = data;
        self
    }
    pub fn set_id(mut self, id: Uid) -> Self {
        self.node.id = id;
        self
    }
    pub fn add_label(mut self, label: String) -> Self {
        self.node.add_labels.insert(label);
        self
    }
    pub fn set_temp_id(mut self, temp_id: Uid) -> Self {
        self.node.temp_id = Some(temp_id);
        self
    }
}

pub struct BlueUpdate<'a, T: GraphTraits, E: GraphTraits> {
    node: UpdateNode<T, E>,
    blueprint: Rc<RefCell<&'a mut BuildBlueprint<'a, T, E>>>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueUpdate<'a, T, E> {
    pub fn update_data(mut self, data: T) -> Self {
        self.node.replacement_data = Some(data);
        self
    }

    pub fn add_label(mut self, label: String) -> Self {
        self.node
            .add_labels
            .get_or_insert_with(|| HashSet::new())
            .insert(label);
        self
    }

    pub fn remove_label(mut self, label: String) -> Self {
        self.node
            .remove_labels
            .get_or_insert_with(|| HashSet::new())
            .insert(label);
        self
    }

    pub fn remove_edge(mut self, edge: EdgeFinder<E>) -> Self {
        self.node
            .remove_edges
            .get_or_insert_with(|| HashSet::new())
            .insert(edge);
        self
    }
}

trait BlueNodeOperator {}

impl<'a, T: GraphTraits, E: GraphTraits> BlueNodeOperator for BlueNew<'a, T, E> {}
impl<'a, T: GraphTraits, E: GraphTraits> BlueNodeOperator for BlueUpdate<'a, T, E> {}
pub trait AddBlueprintEdges<'a, T: GraphTraits, E: GraphTraits> {
    fn add_edge_existing<F>(self, direction: EdgeDir, edge_type: E, id: Uid, f: F) -> Self
    where
        F: FnOnce(BlueUpdate<'a, T, E>) -> BlueUpdate<'a, T, E>;
    fn add_edge_new<F>(self, direction: EdgeDir, edge_type: E, f: F) -> Self
    where
        F: FnOnce(BlueNew<'a, T, E>) -> BlueNew<'a, T, E>;
    fn add_edge_temp(self, direction: EdgeDir, edge_type: E, temp_id: Uid) -> Self;
}

macro_rules! implement_add_blueprint_edges {
    ($type:ty) => {
        impl<'a, T: GraphTraits, E: GraphTraits> AddBlueprintEdges<'a, T, E> for $type {
            fn add_edge_new<F>(self, direction: EdgeDir, edge_type: E, f: F) -> Self
            where
                F: FnOnce(BlueNew<'a, T, E>) -> BlueNew<'a, T, E>,
            {
                let edge_node = f(BlueNew {
                    node: NewNode::new(),
                    blueprint: Rc::clone(&self.blueprint),
                });
                if let Some(temp_id) = edge_node.node.temp_id {
                    self.blueprint
                        .borrow_mut()
                        .temp_id_map
                        .insert(temp_id, edge_node.node.id);
                }

                let this_edge = EdgeDescriptor {
                    direction: direction,
                    edge_type,
                    host: self.node.id,
                    target: edge_node.node.id.clone(),
                    render_responsible: false,
                };
                self.blueprint.borrow_mut().add_node(edge_node.node);
                self.blueprint
                    .borrow_mut()
                    .add_edge(this_edge.invert_drop_render());
                self.blueprint.borrow_mut().add_edge(this_edge);

                self
            }

            fn add_edge_existing<F>(self, direction: EdgeDir, edge_type: E, id: Uid, f: F) -> Self
            where
                F: FnOnce(BlueUpdate<'a, T, E>) -> BlueUpdate<'a, T, E>,
            {
                let edge_node = f(BlueUpdate {
                    node: UpdateNode::new(id),
                    blueprint: Rc::clone(&self.blueprint),
                });

                let this_edge = EdgeDescriptor {
                    direction: direction,
                    edge_type,
                    host: self.node.id,
                    target: edge_node.node.id.clone(),
                    render_responsible: false,
                };
                self.blueprint.borrow_mut().update_node(edge_node.node);
                self.blueprint
                    .borrow_mut()
                    .add_edge(this_edge.invert_drop_render());
                self.blueprint.borrow_mut().add_edge(this_edge);

                self
            }

            fn add_edge_temp(self, direction: EdgeDir, edge_type: E, temp_id: Uid) -> Self {
                let this_edge = EdgeDescriptor {
                    direction: direction,
                    edge_type,
                    host: self.node.id,
                    target: temp_id,
                    render_responsible: false,
                };
                // Not adding inverted edge to reduce the number of edges that need to be checked
                // When this edge is found in the temp_edge finalization step, the inverted edge needs to be added
                self.blueprint.borrow_mut().temp_edges.insert(this_edge);
                self
            }
        }
    };
}

implement_add_blueprint_edges!(BlueNew<'a, T, E>);
implement_add_blueprint_edges!(BlueUpdate<'a, T, E>);

#[cfg(test)]
mod tests {
    use crate::{blueprint::new_node::NewNode, graph::graph::Graph, EdgeDir};

    use super::{AddBlueprintEdges, BuildBlueprint};

    #[test]
    fn test() {
        let graph = Graph::new();
        let mut build_blueprint = BuildBlueprint::<String, String>::new(&graph);

        let initial_node = NewNode::<String, String>::new();
        {
            let mut blue_new = build_blueprint
                .start_with_new_node(initial_node)
                .unwrap()
                .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                    blue_new
                        .set_data("data".into())
                        .add_label("label".into())
                        .set_temp_id(123)
                        .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                            blue_new
                                .set_data("data".into())
                                .add_label("label".into())
                                .set_temp_id(1223)
                        })
                });
            println!("{:?}", blue_new.blueprint);
            panic!("test")
        }
    }
}
