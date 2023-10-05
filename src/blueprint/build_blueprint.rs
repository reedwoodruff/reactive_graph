use std::cell::RefCell;

use im::{HashMap, HashSet};
use leptos::*;

use crate::{
    graph::graph::Graph, EdgeDescriptor, EdgeDir, EdgeFinder, GraphError, GraphTraits, Uid,
};

use super::{
    new_node::{NewNode, TempId},
    update_node::UpdateNode,
};

#[derive(Debug, Clone, Eq, PartialEq)]

pub struct FinalizedBlueprint<T: GraphTraits, E: GraphTraits> {
    pub new_nodes: HashMap<Uid, NewNode<T, E>>,
    pub update_nodes: HashMap<Uid, UpdateNode<T, E>>,
    pub delete_nodes: HashSet<Uid>,
}
pub struct BuildBlueprint<'a, T: GraphTraits, E: GraphTraits> {
    new_nodes: RefCell<HashMap<Uid, NewNode<T, E>>>,
    update_nodes: RefCell<HashMap<Uid, UpdateNode<T, E>>>,
    delete_nodes: RefCell<HashSet<Uid>>,
    temp_edges: RefCell<HashSet<EdgeDescriptor<E>>>,
    temp_id_map: RefCell<HashMap<TempId, Uid>>,
    errors: RefCell<Vec<GraphError>>,
    graph: &'a Graph<T, E>,
}

impl<'a, 'b: 'a, T: GraphTraits, E: GraphTraits> BuildBlueprint<'a, T, E> {
    pub fn new(graph: &'a Graph<T, E>) -> Self {
        Self {
            new_nodes: RefCell::new(HashMap::new()),
            update_nodes: RefCell::new(HashMap::new()),
            delete_nodes: RefCell::new(HashSet::new()),
            temp_edges: RefCell::new(HashSet::new()),
            temp_id_map: RefCell::new(HashMap::new()),
            errors: RefCell::new(Vec::new()),
            graph,
        }
    }

    fn add_node(&self, node: NewNode<T, E>) {
        let mut new_nodes_mut = self.new_nodes.borrow_mut();
        if let Some(existing_entry) = new_nodes_mut.get_mut(&node.id) {
            let result = existing_entry.merge(node);
            if let Err(error) = result {
                self.errors.borrow_mut().push(error);
            }
        } else {
            new_nodes_mut.insert(node.id, node);
        }
    }

    pub fn start_with_new_node(&'b self, node: NewNode<T, E>) -> BlueNew<'a, T, E> {
        self.add_node(node.clone());
        BlueNew {
            node,
            blueprint: self,
        }
    }

    fn update_node(&self, node: UpdateNode<T, E>) {
        if self.graph.nodes.get(&node.id).is_none() {
            self.errors.borrow_mut().push(GraphError::Blueprint(
                format!("update_node: node not found, ID: {:?}", node.id),
            ));
        }

        let mut update_nodes_mut = self.update_nodes.borrow_mut();
        if let Some(existing_entry) = update_nodes_mut.get_mut(&node.id) {
            let result = existing_entry.merge(node);
            if let Err(error) = result {
                self.errors.borrow_mut().push(error);
            }
        } else {
            update_nodes_mut.insert(node.id, node);
        }
    }

    pub fn start_with_update_node(&'b self, node: UpdateNode<T, E>) -> BlueUpdate<'b, T, E> {
        self.update_node(node.clone());
        BlueUpdate {
            node,
            blueprint: self,
        }
    }

    pub fn delete_node(&self, node_id: Uid) {
        let graph_node = self.graph.nodes.get(&node_id);
        if let Some(graph_node) = graph_node {
            graph_node
                .0
                .incoming_edges
                .get()
                .iter()
                .for_each(|(_edge_type, edges)| {
                    edges.iter().for_each(|edge| {
                        self.remove_edge(edge.clone().invert_drop_render());
                    })
                });
            graph_node
                .0
                .outgoing_edges
                .get()
                .iter()
                .for_each(|(_edge_type, edges)| {
                    edges.iter().for_each(|edge| {
                        self.remove_edge(edge.clone().invert_drop_render());
                    })
                });
        } else {
            self.errors.borrow_mut().push(GraphError::Blueprint(
                format!("delete_node: node not found, ID: {:?}", node_id),
            ));
        }
        self.delete_nodes.borrow_mut().insert(node_id);
    }

    fn add_edge(&self, edge: EdgeDescriptor<E>) {
        // If the node exists in the graph, add the edge to the update node
        if self.graph.nodes.get(&edge.host).is_some() {
            let mut update_nodes_mut = self.update_nodes.borrow_mut();
            let update_node = update_nodes_mut
                .entry(edge.host)
                .or_insert_with(|| UpdateNode::new(edge.host));

            if update_node.add_edges.is_none() {
                update_node.add_edges = Some(HashSet::new());
            }
            update_node.add_edges.as_mut().unwrap().insert(edge);
            return;
        }

        // Otherwise we assume it's a new node
        self.new_nodes
            .borrow_mut()
            .entry(edge.host)
            .or_insert_with(|| {
                let new_node = NewNode::new();
                new_node.set_id(edge.host)
            })
            .add_edges
            .insert(edge);
    }

    fn remove_edge(&self, edge: EdgeDescriptor<E>) {
        if self.graph.nodes.get(&edge.host).is_none() {
            self.errors.borrow_mut().push(GraphError::Blueprint(
                format!("remove_edge: node not found, ID: {:?}", edge.host),
            ));
        }
        let mut update_nodes_mut = self.update_nodes.borrow_mut();
        let update_node = update_nodes_mut
            .entry(edge.host)
            .or_insert_with(|| UpdateNode::new(edge.host));

        if update_node.remove_edges.is_none() {
            update_node.remove_edges = Some(HashSet::new());
        }
        update_node.remove_edges.as_mut().unwrap().insert(edge);
    }

    fn update_temp_ids(&self) {
        for edge in self.temp_edges.borrow().iter() {
            let updated_edge = EdgeDescriptor {
                target: *self.temp_id_map.borrow().get(&edge.target).unwrap(),
                ..edge.clone()
            };
            self.add_edge(updated_edge.invert_drop_render());
            self.add_edge(updated_edge);
        }
    }

    pub fn finalize(self) -> Result<FinalizedBlueprint<T, E>, Vec<GraphError>> {
        self.update_temp_ids();
        let errors = self.errors.clone().into_inner();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(FinalizedBlueprint {
            delete_nodes: self.delete_nodes.take(),
            new_nodes: self.new_nodes.take(),
            update_nodes: self.update_nodes.take(),
        })
    }
}

pub struct BlueNew<'a, T: GraphTraits, E: GraphTraits> {
    node: NewNode<T, E>,
    pub blueprint: &'a BuildBlueprint<'a, T, E>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueNew<'a, T, E> {
    pub fn set_data(mut self, data: T) -> Self {
        self.node.data = data;
        self.blueprint.add_node(self.node.clone());
        self
    }
    pub fn set_id(mut self, id: Uid) -> Self {
        self.node.id = id;
        self.blueprint.add_node(self.node.clone());
        self
    }
    pub fn add_label(mut self, label: String) -> Self {
        self.node.add_labels.insert(label);
        self.blueprint.add_node(self.node.clone());
        self
    }
    pub fn set_temp_id(mut self, temp_id: Uid) -> Self {
        self.node.temp_id = Some(temp_id);
        self.blueprint.add_node(self.node.clone());
        self
    }
}

pub struct BlueUpdate<'a, T: GraphTraits, E: GraphTraits> {
    node: UpdateNode<T, E>,
    blueprint: &'a BuildBlueprint<'a, T, E>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueUpdate<'a, T, E> {
    pub fn update_data(mut self, data: T) -> Self {
        self.node.replacement_data = Some(data);
        self.blueprint.update_node(self.node.clone());
        self
    }

    pub fn add_label(mut self, label: String) -> Self {
        self.node
            .add_labels
            .get_or_insert_with(HashSet::new)
            .insert(label);
        self.blueprint.update_node(self.node.clone());
        self
    }

    pub fn remove_label(mut self, label: String) -> Self {
        self.node
            .remove_labels
            .get_or_insert_with(HashSet::new)
            .insert(label);

        self.blueprint.update_node(self.node.clone());
        self
    }

    pub fn remove_edge(mut self, mut edge_finder: EdgeFinder<E>) -> Self {
        if let Some(host) = edge_finder.host {
            if host != self.node.id {
                self.blueprint.errors.borrow_mut().push(GraphError::Blueprint(
                format!(
                    "remove_edge: edge host node id does not match update node id\nEdge Host Node ID: {:?}\nUpdate Node ID: {:?}",
                    host, self.node.id
                ),
            ));
            }
        } else {
            edge_finder.host = Some(self.node.id);
        }

        // Find the edge(s) in the graph
        let graph_node = self
            .blueprint
            .graph
            .nodes
            .get(&self.node.id)
            .unwrap()
            .0
            .clone();

        let found_edges = graph_node.search_for_edge(&edge_finder);

        if found_edges.is_none() {
            self.blueprint
                .errors
                .borrow_mut()
                .push(GraphError::Blueprint(
                    format!(
                        "remove_edge: edge not found\nEdge Finder: {:?}",
                        edge_finder
                    ),
                ));
        }

        if let Some(found_edges) = found_edges {
            for found_edge in found_edges {
                // Update the blueprint immediately with the inverse
                self.blueprint.remove_edge(found_edge.invert_drop_render());

                self.blueprint.remove_edge(found_edge.clone());

                // Update this node with the edges to remove, which will be handled on the completion of the add_edge step
                self.node
                    .remove_edges
                    .get_or_insert_with(HashSet::new)
                    .insert(found_edge);
            }
        }

        self
    }
}

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
            fn add_edge_new<F>(self, dir: EdgeDir, edge_type: E, f: F) -> Self
            where
                F: FnOnce(BlueNew<'a, T, E>) -> BlueNew<'a, T, E>,
            {
                let edge_node = f(BlueNew {
                    node: NewNode::new(),
                    blueprint: self.blueprint,
                });
                if let Some(temp_id) = edge_node.node.temp_id {
                    self.blueprint
                        .temp_id_map
                        .borrow_mut()
                        .insert(temp_id, edge_node.node.id);
                }

                let this_edge = EdgeDescriptor {
                    dir,
                    edge_type,
                    host: self.node.id,
                    target: edge_node.node.id.clone(),
                    render_responsible: false,
                };
                self.blueprint.add_node(edge_node.node);
                self.blueprint.add_edge(this_edge.invert_drop_render());
                self.blueprint.add_edge(this_edge);

                self
            }

            fn add_edge_existing<F>(self, dir: EdgeDir, edge_type: E, id: Uid, f: F) -> Self
            where
                F: FnOnce(BlueUpdate<'a, T, E>) -> BlueUpdate<'a, T, E>,
            {
                let edge_node = f(BlueUpdate {
                    node: UpdateNode::new(id),
                    blueprint: self.blueprint,
                });

                let this_edge = EdgeDescriptor {
                    dir,
                    edge_type,
                    host: self.node.id,
                    target: edge_node.node.id.clone(),
                    render_responsible: false,
                };
                self.blueprint.update_node(edge_node.node);
                self.blueprint.add_edge(this_edge.invert_drop_render());
                self.blueprint.add_edge(this_edge);

                self
            }

            fn add_edge_temp(self, dir: EdgeDir, edge_type: E, temp_id: Uid) -> Self {
                let this_edge = EdgeDescriptor {
                    dir,
                    edge_type,
                    host: self.node.id,
                    target: temp_id,
                    render_responsible: false,
                };
                // Not adding inverted edge to reduce the number of edges that need to be checked
                // When this edge is found in the temp_edge finalization step, the inverted edge needs to be added
                self.blueprint.temp_edges.borrow_mut().insert(this_edge);
                self
            }
        }
    };
}

implement_add_blueprint_edges!(BlueNew<'a, T, E>);
implement_add_blueprint_edges!(BlueUpdate<'a, T, E>);

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use im::HashSet;

    use crate::{
        blueprint::{new_node::NewNode, update_node::UpdateNode},
        graph::graph::Graph,
        reactive_node::build_reactive_node::BuildReactiveNode,
        EdgeDescriptor, EdgeDir,
    };

    use super::{AddBlueprintEdges, BuildBlueprint};

    fn setup_graph() -> Graph<String, String> {
        let mut graph = Graph::new();
        let mut edges1 = HashSet::new();
        edges1.insert(EdgeDescriptor {
            dir: EdgeDir::Emit,
            edge_type: "existing_edge_type".into(),
            host: 999,
            target: 998,
            render_responsible: false,
        });
        let (read_reactive_node_1, write_reative_node_1) =
            BuildReactiveNode::<String, String>::new()
                .id(999)
                .data("Existent node 1".into())
                .map_edges_from_bp(&edges1)
                .build();
        let mut edges2 = HashSet::new();
        edges2.insert(EdgeDescriptor {
            dir: EdgeDir::Recv,
            edge_type: "existing_edge_type".into(),
            host: 998,
            target: 999,
            render_responsible: false,
        });
        let (read_reactive_node_2, write_reative_node_2) =
            BuildReactiveNode::<String, String>::new()
                .id(999)
                .data("Existent node 1".into())
                .map_edges_from_bp(&edges2)
                .build();

        graph.nodes.insert(
            999,
            (
                Rc::new(read_reactive_node_1),
                RefCell::new(write_reative_node_1),
            ),
        );
        graph.nodes.insert(
            998,
            (
                Rc::new(read_reactive_node_2),
                RefCell::new(write_reative_node_2),
            ),
        );

        graph
    }

    #[test]
    fn test() {
        let graph = setup_graph();
        let build_blueprint = BuildBlueprint::<String, String>::new(&graph);

        let mut initial_node = NewNode::<String, String>::new();
        initial_node.id = 42;
        initial_node.data = "Initial Node".into();

        build_blueprint
            .start_with_new_node(initial_node)
            .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                blue_new
                    .set_data("data".into())
                    .add_label("label".into())
                    .set_temp_id(1)
                    .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                        blue_new
                            .set_data("data".into())
                            .add_label("label".into())
                            .set_temp_id(2)
                    })
            });
        let mut initial_node = NewNode::<String, String>::new();
        initial_node.id = 43;
        initial_node.data = "Coming from the other side".into();
        build_blueprint
            .start_with_new_node(initial_node)
            .add_edge_temp(EdgeDir::Recv, "edge_type".into(), 1);
        build_blueprint.delete_node(998);
        build_blueprint
            .start_with_update_node(UpdateNode::new(999))
            .update_data("I have changed".into())
            .add_edge_temp(EdgeDir::Emit, "edge_type".into(), 1);

        let build_blueprint = build_blueprint.finalize().unwrap();

        for node in build_blueprint.new_nodes.values() {
            println!("+ New Node {:?}", node.id);
            if node.temp_id.is_some() {
                println!("  + Temp ID: {:?}", node.temp_id);
            }
            if node.data != String::default() {
                println!("  + Data: {:?}", node.data);
            }
            for edge in node.add_edges.iter() {
                println!("  + Edge: {:?}", edge);
            }
        }
        for node in build_blueprint.update_nodes.values() {
            println!("~ Update Node {:?}", node.id);
            if node.replacement_data.is_some() {
                println!("  ~ Data: {:?}", node.replacement_data);
            }
            for edge in node.add_edges.iter() {
                println!("  + Edge: {:?}", edge);
            }
            for edge in node.remove_edges.iter() {
                println!("  - Edge: {:?}", edge);
            }
            for label in node.add_labels.iter() {
                println!("  + Label: {:?}", label);
            }
            for label in node.remove_labels.iter() {
                println!("  - Label: {:?}", label);
            }
        }
        for node in build_blueprint.delete_nodes.iter() {
            println!("- Delete Node {:?}", node);
        }
        panic!("test");
    }
}
