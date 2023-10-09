use std::{borrow::BorrowMut, cell::RefCell, hash::Hash};

use im::{hashset, HashMap, HashSet};
use leptos::{logging::log, *};

use crate::{
    graph::view_graph::ViewGraph, EdgeDescriptor, EdgeDir, EdgeFinder, GraphError, GraphTraits, Uid,
};

use super::{
    new_node::{NewNode, TempId},
    update_node::UpdateNode,
};

#[derive(Debug, Clone, Eq, PartialEq)]

pub struct AllowedRenderEdgeSpecifier<E: GraphTraits> {
    pub edge_type: E,
    pub direction: EdgeDir,
}

pub struct FinalizedBlueprint<T: GraphTraits, E: GraphTraits> {
    pub new_nodes: HashMap<Uid, NewNode<T, E>>,
    pub update_nodes: HashMap<Uid, UpdateNode<T, E>>,
    pub delete_nodes: HashSet<Uid>,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuildBlueprint<T: GraphTraits, E: GraphTraits> {
    new_nodes: RefCell<HashMap<Uid, NewNode<T, E>>>,
    update_nodes: RefCell<HashMap<Uid, UpdateNode<T, E>>>,
    delete_nodes: RefCell<HashSet<Uid>>,
    entry_edges: RefCell<HashSet<EdgeDescriptor<E>>>,
    // Bool represents whether the node is_new
    temp_edges: RefCell<HashSet<(EdgeDescriptor<E>, bool)>>,
    remove_edge_finders: RefCell<HashSet<EdgeFinder<E>>>,
    removed_render_edges: RefCell<HashSet<EdgeDescriptor<E>>>,
    pub temp_id_map: RefCell<HashMap<TempId, Uid>>,
    errors: RefCell<Vec<GraphError>>,
    // graph: &'a ViewGraph<T, E>,
}

impl<'a, 'b: 'a, T: GraphTraits, E: GraphTraits> BuildBlueprint<T, E> {
    pub fn new() -> Self {
        Self {
            new_nodes: RefCell::new(HashMap::new()),
            update_nodes: RefCell::new(HashMap::new()),
            delete_nodes: RefCell::new(HashSet::new()),
            entry_edges: RefCell::new(HashSet::new()),
            temp_edges: RefCell::new(HashSet::new()),
            temp_id_map: RefCell::new(HashMap::new()),
            remove_edge_finders: RefCell::new(HashSet::new()),
            removed_render_edges: RefCell::new(HashSet::new()),
            errors: RefCell::new(Vec::new()),
        }
    }

    fn add_entry_edge(&self, edge: EdgeDescriptor<E>) {
        self.entry_edges.borrow_mut().insert(edge);
    }

    fn remove_new_node(&self, id: Uid) {
        let _result = self.new_nodes.borrow_mut().remove(&id);
        // if result.is_none() {
        //     self.errors.borrow_mut().push(GraphError::Blueprint(format!(
        //         "Attempted to remove new node, but it was not found in the new_node list\n ID: {:?}",
        //         id
        //     )));
        // }
    }

    fn add_node(&self, node: NewNode<T, E>) {
        let mut new_nodes_mut = self.new_nodes.borrow_mut();
        if let Some(existing_entry) = new_nodes_mut.get_mut(&node.id) {
            let result = existing_entry.merge(node);
            match result {
                Err(err) => self.errors.borrow_mut().push(err),
                Ok(merged_node) => {
                    new_nodes_mut.insert(merged_node.id, merged_node);
                }
            }
        } else {
            new_nodes_mut.insert(node.id, node);
        }
    }

    pub fn start_with_new_node(&'b self) -> BlueNew<'a, T, E> {
        let new_node = NewNode::new();
        self.add_node(new_node.clone());
        BlueNew {
            node: new_node,
            blueprint: self,
        }
    }

    fn update_node(&self, node: UpdateNode<T, E>) {
        // if self.graph.nodes.get(&node.id).is_none() {
        //     self.errors.borrow_mut().push(GraphError::Blueprint(format!(
        //         "update_node: node not found, ID: {:?}",
        //         node.id
        //     )));
        // }

        let mut update_nodes_mut = self.update_nodes.borrow_mut();
        if let Some(existing_entry) = update_nodes_mut.get_mut(&node.id) {
            let result = existing_entry.merge(node);
            match result {
                Err(error) => self.errors.borrow_mut().push(error),
                Ok(merged_node) => {
                    update_nodes_mut.insert(merged_node.id, merged_node);
                }
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
        self.delete_nodes.borrow_mut().insert(node_id);
    }

    fn finalize_delete_nodes(&self, graph: &ViewGraph<T, E>) {
        for node_id in self.delete_nodes.borrow().iter() {
            let graph_node = graph.nodes.get(&node_id);
            // If any of the edges on the deleted node were rendering another node, add the edge to the deleted_render_edges
            // Remove all existing edges on the deleted node
            if let Some(graph_node) = graph_node {
                graph_node.0.incoming_edges.get_untracked().iter().for_each(
                    |(_edge_type, edges)| {
                        edges.iter().for_each(|edge| {
                            let inverted_edge = edge.invert();
                            if inverted_edge
                                .render_info
                                .clone()
                                .is_some_and(|info| info == EdgeDir::Recv)
                            {
                                self.removed_render_edges
                                    .borrow_mut()
                                    .insert(inverted_edge.clone());
                            }
                            self.remove_edge(inverted_edge);
                        })
                    },
                );
                graph_node.0.outgoing_edges.get_untracked().iter().for_each(
                    |(_edge_type, edges)| {
                        edges.iter().for_each(|edge| {
                            let inverted_edge = edge.clone().invert();

                            if inverted_edge
                                .render_info
                                .clone()
                                .is_some_and(|info| info == EdgeDir::Recv)
                            {
                                self.removed_render_edges
                                    .borrow_mut()
                                    .insert(inverted_edge.clone());
                            }
                            self.remove_edge(inverted_edge);
                        })
                    },
                );
            } else {
                self.errors.borrow_mut().push(GraphError::Blueprint(format!(
                    "delete_node: node not found, ID: {:?}",
                    node_id
                )));
            }
        }
    }

    fn finalize_removed_edges(&self, graph: &ViewGraph<T, E>) {
        for edge_finder in self.remove_edge_finders.borrow().iter() {
            // Find the edge(s) in the graph
            // We are manually setting the host in the BlueUpdate method.
            let graph_node = graph
                .nodes
                .get(&edge_finder.host.as_ref().unwrap().iter().next().unwrap())
                .unwrap()
                .0
                .clone();

            let found_edges = graph_node.search_for_edge(&edge_finder);

            if found_edges.is_none() {
                self.errors.borrow_mut().push(GraphError::Blueprint(format!(
                    "remove_edge: edge not found\nEdge Finder: {:?}",
                    edge_finder
                )));
            }

            if let Some(found_edges) = found_edges {
                // If the edge was rendering this node or the node on the other end, add the edge to the removed_render_edges to be handled later
                // Remove the edge
                for found_edge in found_edges {
                    // Update the blueprint immediately with the inverse
                    let inverted_edge = found_edge.clone().invert();
                    if inverted_edge
                        .render_info
                        .clone()
                        .is_some_and(|info| info == EdgeDir::Recv)
                    {
                        self.removed_render_edges
                            .borrow_mut()
                            .insert(inverted_edge.clone());
                    }

                    self.remove_edge(inverted_edge);

                    if found_edge
                        .render_info
                        .clone()
                        .is_some_and(|info| info == EdgeDir::Recv)
                    {
                        self.removed_render_edges
                            .borrow_mut()
                            .insert(found_edge.clone());
                    }

                    self.remove_edge(found_edge);
                }
            }
        }
    }

    fn add_edge(&self, edge: EdgeDescriptor<E>, is_new: bool) {
        if is_new {
            self.new_nodes
                .borrow_mut()
                .entry(edge.host)
                .or_insert_with(|| {
                    let new_node = NewNode::new();
                    new_node.set_id(edge.host)
                })
                .add_edges
                .insert(edge);
        } else {
            let mut update_nodes_mut = self.update_nodes.borrow_mut();
            let update_node = update_nodes_mut
                .entry(edge.host)
                .or_insert_with(|| UpdateNode::new(edge.host));

            if update_node.add_edges.is_none() {
                update_node.add_edges = Some(HashSet::new());
            }
            update_node.add_edges.as_mut().unwrap().insert(edge);
        }
    }

    fn remove_edge(&self, edge: EdgeDescriptor<E>) {
        // if self.graph.nodes.get(&edge.host).is_none() {
        //     self.errors.borrow_mut().push(GraphError::Blueprint(format!(
        //         "remove_edge: node not found, ID: {:?}",
        //         edge.host
        //     )));
        // }
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
        for (edge, is_new) in self.temp_edges.borrow().iter() {
            let updated_edge = EdgeDescriptor {
                target: *self.temp_id_map.borrow().get(&edge.target).unwrap(),
                ..edge.clone()
            };
            // The inverted edge will always be is_new because it represents a node which had a temp_id
            self.add_edge(updated_edge.invert(), true);

            // Temp edges which come from existing edges are entry points
            if !is_new {
                self.entry_edges.borrow_mut().insert(updated_edge.invert());
            }
            self.add_edge(updated_edge, *is_new);
        }
    }

    fn set_render_edges_for_new_nodes(
        &self,
        valid_edge_types: Option<HashSet<E>>,
        entry_point_temp_id: Option<TempId>,
    ) {
        println!("----- Starting set render edges -----");
        let mut starting_nodes: HashSet<NewNode<T, E>> = HashSet::new();
        let mut is_entry_point = false;
        println!("entry_edges: {:?}", self.entry_edges.borrow());

        // TODO: ensure that none of the entry edges point to an existing node which is:
        //  1. Being deleted explicitly
        //  2. Being deleted implicitly (by being rendered in a branch which is irreconcilably broken by a deleted node or render edge)
        if !self.entry_edges.borrow().is_empty() {
            println!("entry_edges: {:?}", self.entry_edges.borrow());
            let mut valid_entry_edges: HashSet<EdgeDescriptor<E>> =
                self.entry_edges.borrow().clone();
            valid_entry_edges.retain(|edge| {
                if let Some(valid_edge_types) = &valid_edge_types {
                    valid_edge_types.contains(&edge.edge_type)
                } else {
                    true
                }
            });

            for edge in valid_entry_edges.iter() {
                let node = self.new_nodes.borrow().clone();
                let node = node.get(&edge.host).clone().unwrap();
                // If the potential entry node does not already have a set render edge, use this edge as the entry edge

                if node.get_render_edge().is_none() {
                    let updated_new_node = node.update_edge_render_info(edge, Some(EdgeDir::Recv));
                    self.new_nodes
                        .borrow_mut()
                        .insert(node.id, updated_new_node.clone());

                    let updated_update_node = self
                        .update_nodes
                        .borrow()
                        .get(&edge.target)
                        .unwrap()
                        .update_edge_render_info(&edge.invert(), Some(EdgeDir::Emit))
                        .unwrap();

                    self.update_nodes
                        .borrow_mut()
                        .insert(edge.target, updated_update_node);
                    starting_nodes.insert(updated_new_node);
                    is_entry_point = true;
                }
                // If it already has a render_edge, then if that render_edge is to an existing node, count that existing edge as an entry edge
                // The other alternative is that a user has manually set a render_edge to another new_node, in which case it is not an entry edge
                else {
                    let edge_is_entry_point =
                        self.update_nodes.borrow().get(&edge.target).is_some();
                    if edge_is_entry_point {
                        starting_nodes.insert(node.clone());
                        is_entry_point = true;
                    }
                }
            }
        } else if let Some(starting_id) = entry_point_temp_id {
            let starting_id = self.temp_id_map.borrow().get(&starting_id).cloned();
            if starting_id.is_none() {
                self.errors.borrow_mut().push(GraphError::Blueprint(format!(
                    "set_render_edges: entry_point_temp_id not found in temp_id_map\nTemp Id: {:?}",
                    starting_id
                )));
                return;
            }
            let starting_id = starting_id.unwrap();
            println!("starting_id: {:?}", starting_id);
            // starting_nodes.insert(*self.temp_id_map.borrow().get(&starting_id).unwrap());
            let node = self.new_nodes.borrow();
            let node = node.get(&starting_id).unwrap();
            if node.get_render_edge().is_some() {
                self.errors.borrow_mut().push(GraphError::Blueprint(format!(
                    "set_render_edges: entry_point cannot have a render edge\nNode Id: {:?}",
                    node.id
                )));
                return;
            }
            starting_nodes.insert(node.clone());
            is_entry_point = true;
        } else {
            println!("No valid entry edges or starting id");
            self.errors.borrow_mut().push(GraphError::Blueprint(
                "set_render_edges: no entry points found".to_string(),
            ));
            return;
        };

        if !is_entry_point {
            self.errors.borrow_mut().push(GraphError::Blueprint(format!(
                "No entry point was found for the specified changes",
            )));
            return;
        }

        println!("starting_nodes:");
        for node in starting_nodes.clone() {
            println!("{:?}", node);
        }

        let mut all_connected_nodes: HashSet<Uid> =
            starting_nodes.iter().map(|node| node.id).collect();
        let mut newly_connected_nodes = starting_nodes;
        let mut remaining_nodes = self
            .new_nodes
            .borrow()
            .clone()
            .iter()
            .map(|(id, _node)| *id)
            .collect::<HashSet<Uid>>()
            .difference(all_connected_nodes.clone());

        while remaining_nodes.len() > 0 {
            if newly_connected_nodes.is_empty() {
                self.errors.borrow_mut().push(GraphError::Blueprint(
                    "could not find render edges for all new nodes".to_string(),
                ));
                return;
            }

            let loop_nodes = newly_connected_nodes.clone();
            newly_connected_nodes.clear();

            for node in loop_nodes {
                let mut edge_finder = EdgeFinder::new()
                    .target(remaining_nodes.clone())
                    .direction(EdgeDir::Emit)
                    .render_info(None)
                    .match_all();
                if let Some(valid_edge_types) = &valid_edge_types {
                    edge_finder = edge_finder.edge_type(valid_edge_types.clone());
                }

                let render_edges = node.find_edges(edge_finder);
                for new_render_edge in render_edges {
                    let target_node = self.new_nodes.borrow().clone();
                    let target_node = target_node.get(&new_render_edge.target).unwrap();

                    if let Some(existing_render_edge) = target_node.get_render_edge() {
                        if all_connected_nodes.contains(&existing_render_edge.target) {
                            newly_connected_nodes.insert(target_node.clone());
                            all_connected_nodes.insert(target_node.id);
                            remaining_nodes.remove(&target_node.id);
                        }
                        continue;
                    }
                    let updated_target_node = target_node
                        .update_edge_render_info(&new_render_edge.invert(), Some(EdgeDir::Recv));

                    let updated_host_node = self
                        .new_nodes
                        .borrow()
                        .get(&node.id)
                        .unwrap()
                        .update_edge_render_info(&new_render_edge, Some(EdgeDir::Emit));

                    self.new_nodes
                        .borrow_mut()
                        .insert(node.id, updated_host_node);
                    self.new_nodes
                        .borrow_mut()
                        .insert(new_render_edge.target, updated_target_node.clone());

                    all_connected_nodes.insert(updated_target_node.id);
                    remaining_nodes.remove(&updated_target_node.id);
                    newly_connected_nodes.insert(updated_target_node);
                }
            }
        }

        println!("all_connected_nodes: {:?}", all_connected_nodes);
    }

    fn set_render_edges(
        &self,
        valid_edge_types: Option<HashSet<E>>,
        entry_point_temp_id: Option<TempId>,
    ) {
        if !self.new_nodes.borrow().is_empty() {
            self.set_render_edges_for_new_nodes(valid_edge_types, entry_point_temp_id);
        }
    }

    pub fn finalize(
        self,
        graph: &ViewGraph<T, E>,
        valid_edge_types: Option<HashSet<E>>,
        entry_point_temp_id: Option<TempId>,
    ) -> Result<FinalizedBlueprint<T, E>, Vec<GraphError>> {
        self.update_temp_ids();
        let errors = self.errors.clone().into_inner();
        if !errors.is_empty() {
            return Err(errors);
        }
        self.finalize_delete_nodes(graph);
        self.finalize_removed_edges(graph);
        self.set_render_edges(valid_edge_types, entry_point_temp_id);
        Ok(FinalizedBlueprint {
            delete_nodes: self.delete_nodes.take(),
            new_nodes: self.new_nodes.take(),
            update_nodes: self.update_nodes.take(),
        })
    }
}

#[derive(Clone)]
pub struct BlueNew<'a, T: GraphTraits, E: GraphTraits> {
    node: NewNode<T, E>,
    pub blueprint: &'a BuildBlueprint<T, E>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueNew<'a, T, E> {
    pub fn set_data(mut self, data: T) -> Self {
        self.node.data = data;
        self.blueprint.add_node(self.node.clone());
        self
    }
    pub fn set_id(mut self, id: Uid) -> Self {
        let prev_id = self.node.id;
        self.node.id = id;
        self.blueprint.remove_new_node(prev_id);
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
        self.blueprint
            .borrow_mut()
            .temp_id_map
            .borrow_mut()
            .insert(temp_id, self.node.id);
        self.blueprint.add_node(self.node.clone());
        self
    }
}

#[derive(Clone)]
pub struct BlueUpdate<'a, T: GraphTraits, E: GraphTraits> {
    node: UpdateNode<T, E>,
    blueprint: &'a BuildBlueprint<T, E>,
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

    // pub fn set_render_edge(mut self, finder: EdgeFinder<E>) -> Self {
    //     todo!()
    // }

    pub fn remove_edge(self, edge_finder: EdgeFinder<E>) -> Self {
        let host_hashset = hashset!(self.node.id);
        let mut edge_finder = edge_finder;

        edge_finder.host = Some(host_hashset);

        self.blueprint
            .remove_edge_finders
            .borrow_mut()
            .insert(edge_finder);

        self
    }
}

pub trait GetIsNew {
    fn get_is_new(&self) -> bool;
}
impl<'b, T: GraphTraits, E: GraphTraits> GetIsNew for BlueNew<'b, T, E> {
    fn get_is_new(&self) -> bool {
        true
    }
}
impl<'b, T: GraphTraits, E: GraphTraits> GetIsNew for BlueUpdate<'b, T, E> {
    fn get_is_new(&self) -> bool {
        false
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
                    render_info: None,
                };
                self.blueprint.add_node(edge_node.node);
                self.blueprint.add_edge(this_edge.invert(), true);
                self.blueprint
                    .add_edge(this_edge.clone(), self.get_is_new());
                if !self.get_is_new() {
                    self.blueprint.add_entry_edge(this_edge.invert());
                }

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

                let this_edge = EdgeDescriptor::new(
                    self.node.id,
                    edge_type,
                    edge_node.node.id.clone(),
                    None,
                    dir,
                );
                self.blueprint.update_node(edge_node.node);
                self.blueprint.add_edge(this_edge.invert(), false);
                self.blueprint
                    .add_edge(this_edge.clone(), self.get_is_new());
                if self.get_is_new() {
                    self.blueprint.add_entry_edge(this_edge);
                }

                self
            }

            fn add_edge_temp(self, dir: EdgeDir, edge_type: E, temp_id: Uid) -> Self {
                let this_edge = EdgeDescriptor {
                    dir,
                    edge_type,
                    host: self.node.id,
                    target: temp_id,
                    render_info: None,
                };
                // Not adding inverted edge to reduce the number of edges that need to be checked
                // When this edge is found in the temp_edge finalization step, the inverted edge needs to be added
                self.blueprint
                    .temp_edges
                    .borrow_mut()
                    .insert((this_edge, self.get_is_new()));
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

    use im::{hashset, HashSet};

    use crate::{
        blueprint::{new_node::NewNode, update_node::UpdateNode},
        graph::view_graph::ViewGraph,
        reactive_node::build_reactive_node::BuildReactiveNode,
        EdgeDescriptor, EdgeDir, EdgeFinder,
    };

    use super::{AddBlueprintEdges, BuildBlueprint};

    fn setup_graph() -> ViewGraph<String, String> {
        let mut graph = ViewGraph::new();
        let mut edges1 = HashSet::new();
        edges1.insert(EdgeDescriptor {
            dir: EdgeDir::Emit,
            edge_type: "existing_edge_type".into(),
            host: 999,
            target: 998,
            render_info: None,
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
            render_info: None,
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
        let build_blueprint = BuildBlueprint::<String, String>::new();

        // let mut initial_node = NewNode::<String, String>::new();
        // initial_node.id = 42;
        // initial_node.data = "Initial Node".into();

        build_blueprint
            .start_with_new_node()
            .set_id(42)
            .set_data("Initial Node".into())
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
        // let mut initial_node = NewNode::<String, String>::new();
        // initial_node.id = 43;
        // initial_node.data = "Coming from the other side".into();
        build_blueprint
            .start_with_new_node()
            .set_id(43)
            .set_data("Coming from the other side".into())
            .add_edge_temp(EdgeDir::Recv, "edge_type".into(), 1);
        // build_blueprint.delete_node(998);
        // build_bluerpint.start_with_
        build_blueprint
            .start_with_update_node(UpdateNode::new(999))
            .update_data("I have changed".into())
            .add_edge_temp(EdgeDir::Emit, "edge_type".into(), 1)
            .remove_edge(EdgeFinder::new().target(hashset!(998)));

        let build_blueprint = build_blueprint.finalize(&graph, None, None).unwrap();

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
