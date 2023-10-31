use std::rc::Rc;

use im::{vector, HashSet, Vector};

use crate::prelude::*;

use super::{
    traversal_descriptor::TraversalDescriptor,
    traversal_edge::{TraversalEdge, UpstreamEdge},
    traversal_node::TraversalNode,
    traversal_step_result::TraversalStepResult,
};

// Note: The current structure is duplicating a lot of the return data
// I.E. The first step contains the entire successful graph, but the second step contains the entire successful graph minus the first step
// This probably isn't the best way to do this, but it does have the benefit of being able to continuously traverse the result from the first step,
// whereas you'd have to manually stitch together the steps to get the entire result graph if each step only contained the relevant nodes
// There is likely some more efficient way to do this with references instead of full clones, but I'm not sure how to do that yet
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) struct TraversalStepRecursiveResult<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub result: Vector<Vector<TraversalStepResult<T, E, A>>>,
    pub visited_all: HashSet<Uid>,
}

pub(crate) fn traverse_step<T: GraphTraits, E: GraphTraits, A: GraphTraits>(
    start_nodes: Vector<TraversalNode<T, E, A>>,
    traversal_descriptor: &TraversalDescriptor<T, E, A>,
    current_step: usize,
    visited_all: HashSet<Uid>,
    get_node: &GetNodeClosure<T, E, A>,
    rolling_result: TraversalStepRecursiveResult<T, E, A>,
) -> Option<TraversalStepRecursiveResult<T, E, A>> {
    if current_step >= traversal_descriptor.steps.len() {
        return Some(rolling_result);
    }
    let mut is_successful_completion = false;

    let mut new_rolling_result = rolling_result.clone();
    for start_node in start_nodes.iter() {
        let _step = traversal_descriptor.steps[current_step].clone();
        let step_item_results = traverse_step_item(
            start_node.clone(),
            current_step,
            visited_all.clone(),
            HashSet::<Uid>::new(),
            0,
            start_node.traversal_index,
            Vector::<TraversalNode<T, E, A>>::new(),
            get_node,
            traversal_descriptor,
            new_rolling_result.clone(),
        );

        if step_item_results.is_none() {
            continue;
        }
        is_successful_completion = true;

        let successful_item_branch = step_item_results.unwrap();
        let mut new_step_results_inner = new_rolling_result.result[current_step].clone();
        new_step_results_inner.push_back(successful_item_branch.into_traversal_step_result());
        new_rolling_result = successful_item_branch.rolling_result;
        new_rolling_result
            .result
            .set(current_step, new_step_results_inner);
    }

    if !is_successful_completion {
        return None;
    }
    Some(new_rolling_result)
}
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) struct TraversalStepItemRecursiveResult<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub result: TraversalNode<T, E, A>,
    pub nodes_visited_this_step: HashSet<Uid>,
    pub exit_nodes: Vector<TraversalNode<T, E, A>>,
    pub rolling_result: TraversalStepRecursiveResult<T, E, A>,
}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> TraversalStepItemRecursiveResult<T, E, A> {
    pub fn into_traversal_step_result(&self) -> TraversalStepResult<T, E, A> {
        TraversalStepResult {
            entry: self.result.clone(),
            endpoints: self.exit_nodes.clone(),
        }
    }
}
pub(crate) fn traverse_step_item<'a, T: GraphTraits, E: GraphTraits, A: GraphTraits>(
    node: TraversalNode<T, E, A>,
    current_step: usize,
    visited_all: HashSet<Uid>,
    visited_step: HashSet<Uid>,
    step_index: usize,
    traversal_index: usize,
    exit_nodes: Vector<TraversalNode<T, E, A>>,
    get_node: &GetNodeClosure<T, E, A>,
    traversal_descriptor: &TraversalDescriptor<T, E, A>,
    rolling_result: TraversalStepRecursiveResult<T, E, A>,
) -> Option<TraversalStepItemRecursiveResult<T, E, A>> {
    let step = Rc::new(
        traversal_descriptor
            .steps
            .get(current_step)
            .unwrap()
            .clone(),
    );
    let step_satisfied = step.count.is_satisfied(step_index);
    let upper_bound_met = step.count.upper_bound_met(step_index);

    let mut new_rolling_result: TraversalStepRecursiveResult<T, E, A> = rolling_result.clone();
    let mut new_exit_nodes = exit_nodes.clone();
    let mut new_node = node.clone();
    let mut is_successful_downstream_result = false;
    let mut is_self_exit_node = false;
    let mut new_visited_all = visited_all.clone();
    let step_clone = step.clone();

    //TODO: Handle cycles
    // if visited_all.contains(&node.node.id) {
    //     // Handle cycle, e.g., return an empty Vec.
    //     todo!();
    // }
    // if visited_step.contains(&node.node.id) {
    //     todo!();
    // }
    let mut new_visited_step = visited_step.clone();
    new_visited_step.insert(node.node.id);

    let matching_edges = node.node.search_for_edge(&step.edge_finder);

    // If the upper bound is met, but the step is not satisfied, then we return none.
    if upper_bound_met && !step_satisfied {
        // No matching edges and step is unsatisfied: return a None to trigger backtracking.
        return None;
    }

    // Checking for the case that this node should be added regardless of further step operations
    // That would be the case if
    // - The step is satisfied on this node and
    // 1. the count is inclusive
    // 2. this is the upper bound of the count
    // 3. the count is exclusive and there are no further matching edges to check
    if step_satisfied
        && (step.count.is_inclusive()
            || upper_bound_met
            || (step.count.is_exclusive() && matching_edges.is_none()))
    {
        let mut new_visited_all_inner = visited_all.clone();
        new_visited_all_inner.extend(new_visited_step.clone());

        let result = traverse_step(
            vector![new_node.clone()],
            traversal_descriptor,
            current_step + 1,
            new_visited_all_inner,
            get_node,
            new_rolling_result.clone(),
        );

        if let Some(successful_completion) = result {
            is_self_exit_node = true;
            new_exit_nodes.push_back(new_node.clone());
            new_rolling_result = successful_completion.clone();
            new_visited_all.extend(successful_completion.visited_all);
        }
    }

    if let Some(matching_edges) = matching_edges {
        if !upper_bound_met {
            // If there are successful results, we need to combine the visited_step and exit_nodes from each result into a single result
            // In addition, we need to add the successful nodes to the current TraversalNode's downstream_edges
            // We are building a result to return to the upstream node

            for edge in matching_edges.iter() {
                let new_trav_node = TraversalNode::new(
                    (get_node)(edge.target).unwrap(),
                    step_index + 1,
                    traversal_index + 1,
                )
                .set_upstream_edge(UpstreamEdge::new(
                    edge.clone(),
                    step_clone.clone(),
                    step_index,
                    traversal_index,
                ));
                let edge_result = traverse_step_item(
                    new_trav_node,
                    current_step,
                    visited_all.clone(),
                    new_visited_step.clone(),
                    step_index + 1,
                    traversal_index + 1,
                    new_exit_nodes.clone(),
                    get_node,
                    traversal_descriptor,
                    new_rolling_result.clone(),
                );
                if let Some(successful_branch) = edge_result {
                    let mut new_visited_all_inner = visited_all.clone();
                    new_visited_all_inner.extend(new_visited_step.clone());

                    let result = traverse_step(
                        successful_branch.exit_nodes.clone(),
                        traversal_descriptor,
                        current_step + 1,
                        new_visited_all_inner,
                        get_node,
                        new_rolling_result.clone(),
                    );
                    if let Some(successful_completion) = result {
                        is_successful_downstream_result = true;

                        new_visited_step.extend(successful_branch.nodes_visited_this_step);
                        // new_exit_nodes.extend(successful_branch.exit_nodes);
                        new_exit_nodes = successful_branch.exit_nodes;
                        new_node = new_node.add_downstream_edge(TraversalEdge::new(
                            edge.clone(),
                            step.clone(),
                            successful_branch.result,
                            step_index,
                            traversal_index,
                        ));

                        new_rolling_result = successful_completion.clone();
                        new_visited_all.extend(successful_completion.visited_all);
                    }
                }
            }
        }
    }

    // If there were no successful results after searching the matching edges, then we need to stop the recursion and return a None to trigger backtracking.
    // This means that the recursive function has run down into the branch and has ultimately found no routes to an exit
    // We propagate that negative finding upward
    if !is_successful_downstream_result && !is_self_exit_node {
        return None;
    }

    Some(TraversalStepItemRecursiveResult {
        result: new_node,
        nodes_visited_this_step: new_visited_step,
        exit_nodes: new_exit_nodes,
        rolling_result: new_rolling_result,
    })
}
