use std::cell::RefCell;

use im::{hashset, vector, HashMap, HashSet, Vector};
use leptos::{logging::log, *};

use crate::graph::view_graph::ViewGraph;
use crate::prelude::*;

use super::{
    delete_node::DeleteNode,
    finalized_update_node::{FinalizedUpdateNode, UpdateNodeReplacementData},
    new_node::{NewNode, TempId},
    update_node::UpdateNode,
};

#[derive(Debug, Clone, Eq, PartialEq)]

pub struct AllowedRenderEdgeSpecifier<E: GraphTraits> {
    pub edge_type: E,
    pub dir: EdgeDir,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuildBlueprint<T: GraphTraits, E: GraphTraits> {
    new_nodes: RefCell<HashMap<Uid, NewNode<T, E>>>,
    update_nodes: RefCell<HashMap<Uid, UpdateNode<T, E>>>,
    delete_nodes: RefCell<HashSet<Uid>>,
    // Should be from the perspective of the renderer -- the new node should be the target
    entry_edges: RefCell<HashSet<EdgeDescriptor<E>>>,
    // Bool represents whether the node is_new
    temp_edges: RefCell<HashSet<(EdgeDescriptor<E>, bool)>>,
    remove_edge_finders: RefCell<HashSet<EdgeFinder<E>>>,
    removed_render_edges: RefCell<HashSet<EdgeDescriptor<E>>>,
    // Stored as NewNodes to facilitate the render path finding algorithm, but these nodes are actually existent.
    // The new edges represent all of the existing edges except for any that have been removed
    displaced_nodes: RefCell<HashMap<Uid, NewNode<T, E>>>,
    // Possible entry edges to the displaced nodes. Should be from the perspective of the renderer
    displaced_entry_edges: RefCell<HashSet<EdgeDescriptor<E>>>,
    pub temp_id_map: RefCell<HashMap<TempId, Uid>>,
    errors: RefCell<Vec<GraphError>>,
}

impl<'a, 'b: 'a, T: GraphTraits, E: GraphTraits> Default for BuildBlueprint<T, E> {
    fn default() -> Self {
        Self::new()
    }
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
            displaced_nodes: RefCell::new(HashMap::new()),
            displaced_entry_edges: RefCell::new(HashSet::new()),
            errors: RefCell::new(Vec::new()),
        }
    }

    fn add_entry_edge(&self, edge: EdgeDescriptor<E>) {
        self.entry_edges.borrow_mut().insert(edge);
    }

    fn remove_new_node(&self, id: Uid) {
        let _result = self.new_nodes.borrow_mut().remove(&id);
    }

    fn add_node(&self, node: NewNode<T, E>) {
        let mut new_nodes_mut = self.new_nodes.borrow_mut();
        if let Some(existing_entry) = new_nodes_mut.get_mut(&node.id) {
            let result = existing_entry.merge_additive(node);
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

    pub fn start_with_update_node(&'b self, id: Uid) -> BlueUpdate<'b, T, E> {
        let update_node = UpdateNode::new(id);
        self.update_node(update_node.clone());
        BlueUpdate {
            node: update_node,
            blueprint: self,
        }
    }

    pub fn delete_node(&self, node_id: Uid) {
        self.delete_nodes.borrow_mut().insert(node_id);
    }

    // Edge should be given from the perspective of the potentially displaced node
    fn find_and_catalog_displaced_nodes<A: GraphTraits>(
        &self,
        graph: &ViewGraph<T, E, A>,
        edge_to_check: EdgeDescriptor<E>,
    ) {
        if edge_to_check
            .render_info
            .clone()
            .is_some_and(|info| info == EdgeDir::Recv)
        {
            let found_node = graph.nodes.get(&edge_to_check.host).unwrap().0.clone();
            let update_node = self
                .update_nodes
                .borrow()
                .as_ref()
                .get(&found_node.id)
                .cloned();
            let mut all_edges_except_current_one: HashSet<EdgeDescriptor<E>> =
                found_node.convert_all_edges_to_hashset();
            all_edges_except_current_one.remove(&edge_to_check);

            // log!("~~~ Edge_to_check: {:?}", edge_to_check.clone());

            if let Some(update_node) = update_node {
                if let Some(add_edges) = update_node.add_edges.clone() {
                    all_edges_except_current_one = all_edges_except_current_one.union(add_edges);
                }
                if let Some(remove_edges) = update_node.remove_edges.clone() {
                    all_edges_except_current_one =
                        all_edges_except_current_one.relative_complement(remove_edges);
                }
            }

            // Converting the existing node into a "NewNode" type in which all "add_edges" represent all remaining valid edges.
            // The rest of the data besides the ID don't matter and won't be used.
            let converted_node = NewNode {
                add_edges: all_edges_except_current_one.clone(),
                id: found_node.id,
                ..NewNode::<T, E>::new()
            };
            // log!("~~~ Converted Node: {:?}", converted_node.clone());
            self.displaced_nodes
                .borrow_mut()
                .insert(found_node.id, converted_node);

            // Check this node's edges to see if it is rendering other nodes which will become displaced
            for edge in all_edges_except_current_one {
                self.find_and_catalog_displaced_nodes(graph, edge.invert());
            }
        }
    }

    fn finalize_delete_nodes<A: GraphTraits>(&self, graph: &ViewGraph<T, E, A>) {
        for node_id in self.delete_nodes.borrow().iter() {
            // Since entry edges should represent everywhere a new node is being connected to an existing node,
            // we should be able to check the entry edges for anywhere the a new node was referencing a deleted node, and remove those edges
            // (This instead of looping through all new nodes searching for edges to the deleted node)
            let found_entry_edges = self
                .entry_edges
                .borrow()
                .iter()
                .filter(|edge| edge.host == *node_id)
                .cloned()
                .collect::<HashSet<EdgeDescriptor<E>>>();

            for found_edge in found_entry_edges {
                self.entry_edges.borrow_mut().remove(&found_edge);
                let mut new_nodes_mut = self.new_nodes.borrow_mut();
                let new_node = new_nodes_mut.get_mut(&found_edge.target).unwrap();
                let found_actual_edges =
                    new_node.find_edges(&EdgeFinder::new().target(*node_id).match_all());
                for found_actual_edge in found_actual_edges {
                    new_node.add_edges.remove(&found_actual_edge);
                }
            }

            // Remove any scheduled updates from this deleted node
            self.update_nodes.borrow_mut().remove(node_id);

            let graph_node = graph.nodes.get(node_id);
            // If any of the edges on the deleted node were rendering another node, add the edge to the deleted_render_edges
            // Remove all existing edges on the deleted node
            if let Some(graph_node) = graph_node {
                graph_node.0.incoming_edges.get_untracked().iter().for_each(
                    |(_edge_type, edges)| {
                        edges.iter().for_each(|edge| {
                            let inverted_edge = edge.invert();

                            self.remove_edge(inverted_edge.clone());
                            self.find_and_catalog_displaced_nodes(graph, inverted_edge);
                        })
                    },
                );
                graph_node.0.outgoing_edges.get_untracked().iter().for_each(
                    |(_edge_type, edges)| {
                        edges.iter().for_each(|edge| {
                            let inverted_edge = edge.clone().invert();

                            self.remove_edge(inverted_edge.clone());
                            self.find_and_catalog_displaced_nodes(graph, inverted_edge)
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

    fn finalize_removed_edges<A: GraphTraits>(&self, graph: &ViewGraph<T, E, A>) {
        for edge_finder in self.remove_edge_finders.borrow().iter() {
            // Find the edge(s) in the graph
            // We are manually setting the host in the BlueUpdate method.
            let graph_node = graph
                .nodes
                .get(edge_finder.host.as_ref().unwrap().iter().next().unwrap())
                .unwrap()
                .0
                .clone();

            let found_edges = graph_node.search_for_edge(edge_finder);

            // log!("graph_node id: {:?}", graph_node.id);
            // log!("graph_node outgoing: {:?}", graph_node.outgoing_edges.get());
            // log!("graph_node incoming: {:?}", graph_node.incoming_edges.get());
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

                    self.find_and_catalog_displaced_nodes(graph, inverted_edge.clone());
                    self.remove_edge(inverted_edge);

                    self.find_and_catalog_displaced_nodes(graph, found_edge.clone());
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
        self.entry_edges.borrow_mut().remove(&edge);
        self.displaced_entry_edges.borrow_mut().remove(&edge);

        let displaced_node = self.displaced_nodes.borrow().get(&edge.host).cloned();
        if let Some(displaced_node) = displaced_node {
            let mut displaced_node_mut = displaced_node.clone();
            displaced_node_mut.add_edges.remove(&edge);
            self.displaced_nodes
                .borrow_mut()
                .insert(displaced_node.id, displaced_node_mut);
        }

        let mut update_nodes_mut = self.update_nodes.borrow_mut();
        let update_node = update_nodes_mut
            .entry(edge.host)
            .or_insert_with(|| UpdateNode::new(edge.host));

        if let Some(add_edges) = update_node.add_edges.as_mut() {
            add_edges.remove(&edge);
        }

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

            // Temp edges which come from existing nodes are entry points
            if !is_new {
                self.add_entry_edge(updated_edge.clone());
            }
            self.add_edge(updated_edge, *is_new);
        }
    }

    fn find_potential_entries_for_displaced_nodes<A: GraphTraits>(
        &self,
        graph: &ViewGraph<T, E, A>,
    ) {
        for (id, displaced_node) in self.displaced_nodes.borrow().iter() {
            displaced_node.add_edges.iter().for_each(|edge| {
                if graph.nodes.get(&edge.target).is_some()
                    && !self.delete_nodes.borrow().contains(id)
                    && self.displaced_nodes.borrow().get(&edge.target).is_none()
                {
                    self.displaced_entry_edges
                        .borrow_mut()
                        .insert(edge.invert());
                }
            });
        }
    }

    fn set_render_edges<A: GraphTraits>(
        &self,
        valid_render_edge_finders: Vector<EdgeFinder<E>>,
        entry_point_temp_id: Option<TempId>,
        graph: &ViewGraph<T, E, A>,
    ) {
        let flipped_valid_edge_finders: Vector<EdgeFinder<E>> = valid_render_edge_finders
            .iter()
            .map(|finder| finder.invert())
            .collect();

        let combined_entry_edges: HashSet<EdgeDescriptor<E>> = self
            .entry_edges
            .borrow()
            .iter()
            .chain(self.displaced_entry_edges.borrow().iter())
            .filter(|&edge| {
                valid_render_edge_finders
                    .iter()
                    .any(|finder| finder.matches(edge))
            })
            .cloned()
            .collect();

        let mut ranked_connection_possibilities: HashMap<u64, HashSet<EdgeDescriptor<E>>> =
            combined_entry_edges
                .iter()
                .map(|edge| {
                    let order_value = flipped_valid_edge_finders
                        .iter()
                        .position(|finder| finder.matches(edge))
                        .unwrap_or(usize::MAX);

                    (order_value as u64, hashset!(edge.clone()))
                })
                .collect();

        let mut combined_uncertain_render_nodes = self.new_nodes.borrow().clone();
        combined_uncertain_render_nodes.extend(self.displaced_nodes.borrow().clone());

        let mut all_connected_nodes: HashSet<Uid> = HashSet::new();
        let mut newly_connected_nodes: HashSet<NewNode<T, E>> = HashSet::new();
        let mut remaining_nodes = combined_uncertain_render_nodes
            .iter()
            .map(|(id, _node)| *id)
            .collect::<HashSet<Uid>>();

        if let Some(entry_point_temp_id) = entry_point_temp_id {
            let starting_id = self.temp_id_map.borrow().get(&entry_point_temp_id).cloned();
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

            let node = combined_uncertain_render_nodes
                .get(&starting_id)
                .cloned()
                .unwrap();
            if node.get_render_edge().is_some() {
                self.errors.borrow_mut().push(GraphError::Blueprint(format!(
                    "set_render_edges: entry_point cannot have a render edge\nNode Id: {:?}",
                    node.id
                )));
                return;
            }
            // starting_nodes.insert(node.clone());
            newly_connected_nodes.insert(node.clone());
            all_connected_nodes.insert(node.id);
            remaining_nodes.remove(&node.id);
        }

        while !remaining_nodes.is_empty() {
            for newly_connected_node in newly_connected_nodes.clone() {
                for edge in newly_connected_node.add_edges {
                    if all_connected_nodes.contains(&edge.target) {
                        continue;
                    }
                    let order_value = valid_render_edge_finders
                        .iter()
                        .position(|finder| finder.matches(&edge));

                    if let Some(order_value) = order_value {
                        all_connected_nodes.insert(edge.target);
                        ranked_connection_possibilities
                            .entry(order_value as u64)
                            .or_default()
                            .insert(edge);
                    }
                }
            }
            newly_connected_nodes.clear();

            // If there are no more possible connection edges, check to see if any of the remaining nodes are new nodes
            // If there are new nodes which can't be connected, return an error
            // If there are only displaced nodes which can't be connected, delete them and finish the function
            if ranked_connection_possibilities.is_empty() {
                log!("Finished loop, remaining nodes: {:?}", remaining_nodes);
                for remaining_node in remaining_nodes.clone() {
                    if self.new_nodes.borrow().get(&remaining_node).is_some() {
                        self.errors.borrow_mut().push(GraphError::Blueprint(
                            format!(
                                "could not find render edges for all new nodes\nNode ID: {:?}",
                                remaining_node
                            )
                            .to_string(),
                        ));
                        return;
                    }
                }

                for remaining_node in remaining_nodes {
                    self.delete_nodes.borrow_mut().insert(remaining_node);
                }
                // Should be fine to just call this function again, even though it will do some duplicate work
                // If it's a performance problem we can do a more narrow update
                self.finalize_delete_nodes(graph);

                return;
            }

            // Get the first connection in the highest ranked connection possibilities. Remove it from the list. If it was the last item in the list, remove that rank from the map.
            let (rank, connection_possibilities) = ranked_connection_possibilities
                .iter()
                .next()
                .map(|(rank, connections)| (*rank, connections.clone()))
                .expect("ranked_connection_possibilities should not be empty");

            let connection = connection_possibilities
                .iter()
                .next()
                .cloned()
                .expect("connection_possibilities should not be empty");

            ranked_connection_possibilities
                .entry(rank)
                .and_modify(|set| {
                    set.remove(&connection);
                });

            if ranked_connection_possibilities
                .get(&rank)
                .is_some_and(|list| list.is_empty())
            {
                ranked_connection_possibilities.remove(&rank);
            }

            let target_node = combined_uncertain_render_nodes
                .get(&connection.target)
                .cloned()
                .unwrap();

            if let Some(existing_render_edge) = target_node.get_render_edge() {
                if all_connected_nodes.contains(&existing_render_edge.target) {
                    newly_connected_nodes.insert(target_node.clone());
                    all_connected_nodes.insert(target_node.id);
                    remaining_nodes.remove(&target_node.id);
                }
                continue;
            }

            // Handle new nodes and displaced nodes differently
            let host_is_new = self.new_nodes.borrow().get(&connection.host).is_some();
            let target_is_new = self.new_nodes.borrow().get(&connection.target).is_some();

            //If the same edge already exists, leave it as is to avoid adding and removing the same edge
            // This should only ever be true if both nodes are existing nodes
            let mut contains_same_edge = false;
            let connection_clone = connection.clone();
            let graph_node = graph.nodes.get(&connection.host);
            if let Some(graph_node) = graph_node {
                contains_same_edge = graph_node
                    .0
                    .search_for_edge(
                        &EdgeFinder::new()
                            .host(connection_clone.host)
                            .edge_type(connection_clone.edge_type.clone())
                            .dir(connection_clone.dir)
                            .target(connection_clone.target)
                            .render_info(Some(EdgeDir::Emit)),
                    )
                    .is_some();
            }

            if host_is_new {
                let updated_host_node = self
                    .new_nodes
                    .borrow()
                    .get(&connection.host)
                    .unwrap()
                    .update_edge_render_info(&connection, Some(EdgeDir::Emit));
                self.new_nodes
                    .borrow_mut()
                    .insert(updated_host_node.id, updated_host_node);
            } else if !contains_same_edge {
                let existing_host_node = self.update_nodes.borrow().get(&connection.host).cloned();

                let updated_host_node = if let Some(existing_host_node) = existing_host_node {
                    existing_host_node.update_edge_render_info(&connection, Some(EdgeDir::Emit))
                } else {
                    let new_node = UpdateNode::new(connection.host);
                    new_node.update_edge_render_info(&connection, Some(EdgeDir::Emit))
                };

                self.update_nodes
                    .borrow_mut()
                    .insert(updated_host_node.id, updated_host_node.clone());
            }

            if target_is_new {
                let updated_target_node = self
                    .new_nodes
                    .borrow()
                    .get(&connection.target)
                    .unwrap()
                    .update_edge_render_info(&connection.invert(), Some(EdgeDir::Recv));
                self.new_nodes
                    .borrow_mut()
                    .insert(updated_target_node.id, updated_target_node);
            } else if !contains_same_edge {
                let existing_target_node =
                    self.update_nodes.borrow().get(&connection.target).cloned();
                let prepared_target_node = if let Some(existing_target_node) = existing_target_node
                {
                    existing_target_node
                        .update_edge_render_info(&connection.invert(), Some(EdgeDir::Recv))
                } else {
                    let new_node = UpdateNode::new(connection.target);
                    new_node.update_edge_render_info(&connection.invert(), Some(EdgeDir::Recv))
                };

                self.update_nodes
                    .borrow_mut()
                    .insert(prepared_target_node.id, prepared_target_node.clone());
            }

            all_connected_nodes.insert(connection.target);
            remaining_nodes.remove(&connection.target);
            newly_connected_nodes.insert(target_node);
        }
    }

    pub fn finalize<A: GraphTraits>(
        self,
        graph: &ViewGraph<T, E, A>,
        // Will be chosen with preference to the order they are specified
        // None defaults to all edge types in the Emit direction
        render_edge_types: Option<Vector<AllowedRenderEdgeSpecifier<E>>>,
        entry_point_temp_id: Option<TempId>,
    ) -> Result<FinalizedBlueprint<T, E>, Vec<GraphError>> {
        let valid_render_edge_finders: Vector<EdgeFinder<E>> =
            if let Some(render_edge_types) = &render_edge_types {
                render_edge_types
                    .iter()
                    .map(|edge_type| {
                        EdgeFinder::new()
                            .edge_type(edge_type.edge_type.clone())
                            .dir(edge_type.dir.clone())
                        // .render_info(None)
                    })
                    .collect()
            } else {
                vector![EdgeFinder::new().dir(EdgeDir::Emit)
                // .render_info(None)
                ]
            };

        self.update_temp_ids();

        self.finalize_delete_nodes(graph);

        self.finalize_removed_edges(graph);

        self.find_potential_entries_for_displaced_nodes(graph);

        self.set_render_edges(valid_render_edge_finders, entry_point_temp_id, graph);

        let mut finalized_delete_nodes = HashMap::<Uid, DeleteNode<T, E>>::new();
        for delete_id in self.delete_nodes.take().iter() {
            finalized_delete_nodes.insert(
                *delete_id,
                DeleteNode::from_read_reactive_node(&graph.nodes.get(delete_id).unwrap().0),
            );
        }
        let mut finalized_update_nodes = HashMap::<Uid, FinalizedUpdateNode<T, E>>::new();
        for (id, update_node) in self.update_nodes.take() {
            let finalized_replacement_data =
                update_node
                    .replacement_data
                    .map(|replacement_data| UpdateNodeReplacementData {
                        new_data: replacement_data,
                        prev_data: graph.nodes.get(&id).unwrap().0.data.get_untracked(),
                    });
            finalized_update_nodes.insert(
                id,
                FinalizedUpdateNode {
                    replacement_data: finalized_replacement_data,
                    add_edges: update_node.add_edges,
                    add_labels: update_node.add_labels,
                    id: update_node.id,
                    remove_edges: update_node.remove_edges,
                    remove_labels: update_node.remove_labels,
                },
            );
        }

        let errors = self.errors.clone().into_inner();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(FinalizedBlueprint {
            delete_nodes: finalized_delete_nodes,
            new_nodes: self.new_nodes.take(),
            update_nodes: finalized_update_nodes,
        })
    }
}

#[derive(Clone, Debug)]
pub struct BlueNew<'a, T: GraphTraits, E: GraphTraits> {
    pub node: NewNode<T, E>,
    pub blueprint: &'a BuildBlueprint<T, E>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueNew<'a, T, E> {
    pub fn set_data(&self, data: T) -> Self {
        let new_node = NewNode {
            data,
            ..self.node.clone()
        };
        self.blueprint.add_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }
    pub fn set_id(&self, id: Uid) -> Self {
        let prev_id = self.node.id;
        self.blueprint.remove_new_node(prev_id);
        let new_node = NewNode {
            id,
            ..self.node.clone()
        };
        self.blueprint.add_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }
    pub fn add_label(&self, label: String) -> Self {
        let mut new_labels = self.node.add_labels.clone();
        new_labels.insert(label);
        let new_node = NewNode {
            add_labels: new_labels,
            ..self.node.clone()
        };
        self.blueprint.add_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }
    pub fn set_temp_id(&self, temp_id: Uid) -> Self {
        let new_node = NewNode {
            temp_id: Some(temp_id),
            ..self.node.clone()
        };
        self.blueprint
            .temp_id_map
            .borrow_mut()
            .insert(temp_id, self.node.id);
        self.blueprint.add_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }
}

#[derive(Clone)]
pub struct BlueUpdate<'a, T: GraphTraits, E: GraphTraits> {
    pub node: UpdateNode<T, E>,
    pub blueprint: &'a BuildBlueprint<T, E>,
}

impl<'a, T: GraphTraits, E: GraphTraits> BlueUpdate<'a, T, E> {
    pub fn update_data(&self, data: T) -> Self {
        let new_node = UpdateNode {
            replacement_data: Some(data),
            ..self.node.clone()
        };
        self.blueprint.update_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }

    pub fn add_label(&self, label: String) -> Self {
        let mut new_labels = self.node.add_labels.clone().unwrap_or_default();
        new_labels.insert(label);
        let new_node = UpdateNode {
            add_labels: Some(new_labels),
            ..self.node.clone()
        };
        self.blueprint.update_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }

    pub fn remove_label(&self, label: String) -> Self {
        let mut new_labels = self.node.remove_labels.clone().unwrap_or_default();
        new_labels.insert(label);
        let new_node = UpdateNode {
            remove_labels: Some(new_labels),
            ..self.node.clone()
        };
        self.blueprint.update_node(new_node.clone());
        Self {
            node: new_node,
            ..self.clone()
        }
    }

    pub fn remove_edge(&self, edge_finder: EdgeFinder<E>) -> Self {
        let host_hashset = hashset!(self.node.id);
        let mut edge_finder = edge_finder;

        edge_finder.host = Some(host_hashset);

        self.blueprint
            .remove_edge_finders
            .borrow_mut()
            .insert(edge_finder);

        self.clone()
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
    fn add_edge_existing<F>(&self, direction: EdgeDir, edge_type: E, id: Uid, f: F) -> Self
    where
        F: FnOnce(BlueUpdate<'a, T, E>) -> BlueUpdate<'a, T, E>;
    fn add_edge_new<F>(&self, direction: EdgeDir, edge_type: E, f: F) -> Self
    where
        F: FnOnce(BlueNew<'a, T, E>) -> BlueNew<'a, T, E>;
    fn add_edge_temp(&self, direction: EdgeDir, edge_type: E, temp_id: Uid) -> Self;
}

macro_rules! implement_add_blueprint_edges {
    ($type:ty) => {
        impl<'a, T: GraphTraits, E: GraphTraits> AddBlueprintEdges<'a, T, E> for $type {
            fn add_edge_new<F>(&self, dir: EdgeDir, edge_type: E, f: F) -> Self
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
                    self.blueprint.add_entry_edge(this_edge);
                }

                self.clone()
            }

            fn add_edge_existing<F>(&self, dir: EdgeDir, edge_type: E, id: Uid, f: F) -> Self
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
                    self.blueprint.add_entry_edge(this_edge.invert());
                }

                self.clone()
            }

            fn add_edge_temp(&self, dir: EdgeDir, edge_type: E, temp_id: Uid) -> Self {
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
                self.clone()
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

    use crate::prelude::reactive_node::build_reactive_node::BuildReactiveNode;
    use crate::prelude::reactive_node::last_action::ActionData;
    use crate::prelude::*;
    use crate::{graph::view_graph::ViewGraph, prelude::reactive_node::last_action::LastAction};

    use super::{AddBlueprintEdges, BuildBlueprint, FinalizedBlueprint};

    /// (999) -> (998) -> (999)
    fn manual_setup_graph() -> ViewGraph<String, String, String> {
        let mut graph = ViewGraph::new();
        let mut edges1 = HashSet::new();
        edges1.insert(EdgeDescriptor {
            dir: EdgeDir::Emit,
            edge_type: "existing_edge_type".into(),
            host: 999,
            target: 998,
            render_info: Some(EdgeDir::Emit),
        });
        let (read_reactive_node_1, write_reative_node_1) =
            BuildReactiveNode::<String, String, String>::new()
                .id(999)
                .data("Existent node 1".into())
                .map_edges_from_bp(&edges1)
                .add_last_action(LastAction {
                    action_data: Rc::new(ActionData::new("manual create".to_string())),
                    update_info: None,
                })
                .build();
        let mut edges2 = HashSet::new();
        edges2.insert(EdgeDescriptor {
            dir: EdgeDir::Recv,
            edge_type: "existing_edge_type".into(),
            host: 998,
            target: 999,
            render_info: Some(EdgeDir::Recv),
        });
        edges2.insert(EdgeDescriptor {
            dir: EdgeDir::Emit,
            edge_type: "existing_edge_type".into(),
            host: 998,
            target: 997,
            render_info: Some(EdgeDir::Emit),
        });
        let (read_reactive_node_2, write_reative_node_2) =
            BuildReactiveNode::<String, String, String>::new()
                .id(998)
                .data("Existent node 1".into())
                .map_edges_from_bp(&edges2)
                .add_last_action(LastAction {
                    action_data: Rc::new(ActionData::new("manual create".to_string())),
                    update_info: None,
                })
                .build();
        let mut edges3 = HashSet::new();
        edges3.insert(EdgeDescriptor {
            dir: EdgeDir::Recv,
            edge_type: "existing_edge_type".into(),
            host: 997,
            target: 998,
            render_info: Some(EdgeDir::Recv),
        });
        let (read_reactive_node_3, write_reative_node_3) =
            BuildReactiveNode::<String, String, String>::new()
                .id(997)
                .data("Existent node 1".into())
                .map_edges_from_bp(&edges3)
                .add_last_action(LastAction {
                    action_data: Rc::new(ActionData::new("manual create".to_string())),
                    update_info: None,
                })
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
        graph.nodes.insert(
            997,
            (
                Rc::new(read_reactive_node_3),
                RefCell::new(write_reative_node_3),
            ),
        );

        graph
    }

    fn log_finalize_results(build_blueprint: &FinalizedBlueprint<String, String>) {
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
    }

    #[test]
    fn should_generate_correct_graph_from_only_new_nodes() {
        let mut graph = ViewGraph::new();
        let build_blueprint = BuildBlueprint::<String, String>::new();

        build_blueprint
            .start_with_new_node()
            .set_id(1)
            .set_temp_id(1)
            .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                blue_new
                    .set_id(2)
                    .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                        blue_new.set_id(3)
                    })
            });

        let build_blueprint = build_blueprint
            .finalize::<String>(&graph, None, Some(1))
            .unwrap();

        let action_data = ActionData::new("update".to_string());

        graph.add_nodes(build_blueprint.new_nodes, action_data.clone());
        graph.delete_nodes(build_blueprint.delete_nodes).unwrap();
        graph
            .update_nodes(build_blueprint.update_nodes, action_data.clone())
            .unwrap();

        assert_eq!(graph.nodes.len(), 3);

        assert!(graph
            .nodes
            .get(&1)
            .unwrap()
            .0
            .search_for_edge(
                &EdgeFinder::new()
                    .target(2)
                    .dir(EdgeDir::Emit)
                    .render_info(Some(EdgeDir::Emit))
            )
            .is_some());

        assert!(graph
            .nodes
            .get(&2)
            .unwrap()
            .0
            .search_for_edge(
                &EdgeFinder::new()
                    .target(3)
                    .dir(EdgeDir::Emit)
                    .render_info(Some(EdgeDir::Emit))
            )
            .is_some(),);
        assert!(graph
            .nodes
            .get(&2)
            .unwrap()
            .0
            .search_for_edge(
                &EdgeFinder::new()
                    .target(1)
                    .dir(EdgeDir::Recv)
                    .render_info(Some(EdgeDir::Recv))
            )
            .is_some(),);
        assert!(graph
            .nodes
            .get(&3)
            .unwrap()
            .0
            .search_for_edge(
                &EdgeFinder::new()
                    .target(2)
                    .dir(EdgeDir::Recv)
                    .render_info(Some(EdgeDir::Recv))
            )
            .is_some(),);
    }

    #[test]
    fn should_fail_if_new_node_cannot_be_connected() {
        let graph = ViewGraph::new();
        let build_blueprint = BuildBlueprint::<String, String>::new();

        build_blueprint
            .start_with_new_node()
            .set_id(1)
            .set_temp_id(1)
            .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                blue_new.set_id(2)
            });

        build_blueprint
            .start_with_new_node()
            .set_id(3)
            .set_temp_id(2);

        let build_blueprint = build_blueprint.finalize::<String>(&graph, None, Some(1));

        assert!(build_blueprint.is_err());
    }

    #[test]
    fn should_correctly_attach_new_node_subgraph_to_existing_graph() {
        let mut graph = manual_setup_graph();
        let build_blueprint = BuildBlueprint::<String, String>::new();

        build_blueprint
            .start_with_new_node()
            .set_id(1)
            .set_temp_id(1)
            .add_edge_existing(EdgeDir::Recv, "edge_type".into(), 999, |blue_update| {
                blue_update
            })
            .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                blue_new
                    .set_id(2)
                    .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                        blue_new.set_id(3)
                    })
            });

        let build_blueprint = build_blueprint.finalize(&graph, None, None).unwrap();

        let action_data = ActionData::new("update".to_string());
        graph.add_nodes(build_blueprint.new_nodes, action_data.clone());
        graph.delete_nodes(build_blueprint.delete_nodes).unwrap();
        graph
            .update_nodes(build_blueprint.update_nodes, action_data.clone())
            .unwrap();

        assert!(graph
            .nodes
            .get(&1)
            .unwrap()
            .0
            .search_for_edge(
                &EdgeFinder::new()
                    .target(999)
                    .render_info(Some(EdgeDir::Recv))
            )
            .is_some());
    }

    #[test]
    fn should_delete_existing_nodes_which_are_no_longer_renderable() {
        let graph = manual_setup_graph();
        let build_blueprint = BuildBlueprint::<String, String>::new();

        build_blueprint
            .start_with_update_node(999)
            .remove_edge(EdgeFinder::new().target(998));

        let build_blueprint = build_blueprint.finalize(&graph, None, None).unwrap();
        assert!(build_blueprint.delete_nodes.get(&998).is_some());
        assert!(build_blueprint.delete_nodes.get(&997).is_some());
    }

    #[test]
    fn should_reconnect_existing_nodes_which_were_disconnected_if_possible() {
        let mut graph = manual_setup_graph();
        let build_blueprint = BuildBlueprint::<String, String>::new();

        build_blueprint
            .start_with_new_node()
            .set_id(1)
            .add_edge_existing(EdgeDir::Recv, "edge_type".into(), 999, |blue_update| {
                blue_update.remove_edge(EdgeFinder::new().target(998))
            })
            .add_edge_existing(EdgeDir::Emit, "edge_type".into(), 997, |blue_update| {
                blue_update
            });

        let build_blueprint = build_blueprint.finalize(&graph, None, None).unwrap();
        assert!(build_blueprint.delete_nodes.get(&998).is_some());
        assert!(build_blueprint.delete_nodes.get(&997).is_none());

        log_finalize_results(&build_blueprint);

        let action_data = ActionData::new("update".to_string());
        graph.add_nodes(build_blueprint.new_nodes, action_data.clone());
        graph.delete_nodes(build_blueprint.delete_nodes).unwrap();
        graph
            .update_nodes(build_blueprint.update_nodes, action_data.clone())
            .unwrap();

        assert!(graph
            .nodes
            .get(&997)
            .unwrap()
            .0
            .search_for_edge(&EdgeFinder::new().target(1).render_info(Some(EdgeDir::Recv)))
            .is_some());
    }

    #[test]
    fn should_reconnect_child_existing_nodes_which_were_disconnected_using_new_node_if_possible() {
        let mut graph = manual_setup_graph();
        let build_blueprint = BuildBlueprint::<String, String>::new();

        build_blueprint
            .start_with_new_node()
            .set_id(1)
            .add_edge_existing(EdgeDir::Recv, "edge_type".into(), 999, |blue_update| {
                blue_update.remove_edge(EdgeFinder::new().target(998))
            })
            .add_edge_existing(EdgeDir::Emit, "edge_type".into(), 998, |blue_update| {
                blue_update
            });

        let build_blueprint = build_blueprint.finalize(&graph, None, None).unwrap();
        log_finalize_results(&build_blueprint);

        assert!(build_blueprint.delete_nodes.get(&998).is_none());
        assert!(build_blueprint.delete_nodes.get(&997).is_none());

        let action_data = ActionData::new("update".to_string());
        graph.add_nodes(build_blueprint.new_nodes, action_data.clone());
        graph.delete_nodes(build_blueprint.delete_nodes).unwrap();
        graph
            .update_nodes(build_blueprint.update_nodes, action_data.clone())
            .unwrap();

        assert!(graph
            .nodes
            .get(&998)
            .unwrap()
            .0
            .search_for_edge(&EdgeFinder::new().target(1).render_info(Some(EdgeDir::Recv)))
            .is_some());

        assert!(graph
            .nodes
            .get(&997)
            .unwrap()
            .0
            .search_for_edge(
                &EdgeFinder::new()
                    .target(998)
                    .render_info(Some(EdgeDir::Recv))
            )
            .is_some());
    }
}
