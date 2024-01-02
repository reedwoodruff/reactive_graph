use std::{fmt::Formatter, iter};

use crate::prelude::*;

use super::{
    traversal_executor::{traverse_step, TraversalStepRecursiveResult},
    traversal_step::{TraversalCount, TraversalStep},
};



use im::{HashSet, Vector};



use super::{
    traversal_node::TraversalNode,
    traversal_result::TraversalResult,
    traversal_step_result::TraversalStepResult,
};

#[derive(Clone)]
pub struct TraversalDescriptor<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub start_node: Uid,
    pub steps: Vector<TraversalStep<T, E, A>>,
    pub get_node_closure: GetNodeClosure<T, E, A>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> PartialEq for TraversalDescriptor<T, E, A> {
    fn eq(&self, other: &Self) -> bool {
        self.start_node == other.start_node && self.steps == other.steps
    }
}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Eq for TraversalDescriptor<T, E, A> {}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> std::fmt::Debug
    for TraversalDescriptor<T, E, A>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraversalDescriptor")
            .field("start_node", &self.start_node)
            .field("steps", &self.steps)
            .finish()
    }
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> TraversalDescriptor<T, E, A> {
    pub fn new(start_node: Uid, get_node_closure: GetNodeClosure<T, E, A>) -> Self {
        Self {
            start_node,
            get_node_closure,
            steps: Vector::new(),
        }
    }

    pub fn add_step(&self, edge_finder: EdgeFinder<T, E, A>, count: TraversalCount) -> Self {
        let mut new_steps = self.steps.clone();
        new_steps.push_back(TraversalStep::new(edge_finder, count));
        Self {
            start_node: self.start_node,
            get_node_closure: self.get_node_closure.clone(),
            steps: new_steps,
        }
    }

    pub fn execute(&self) -> Option<TraversalResult<T, E, A>> {
        let get_node = self.get_node_closure.clone();
        let root_trav_node = TraversalNode::new((get_node)(&self.start_node).unwrap(), 0, 0);
        let visited_all: HashSet<Uid> = HashSet::new();

        let initial_result_vector =
            iter::repeat_with(Vector::<TraversalStepResult<T, E, A>>::new)
                .take(self.steps.len())
                .collect::<Vector<Vector<TraversalStepResult<T, E, A>>>>();

        // let mut result = TraversalResult::<T, E, A>::new(self.steps.len());
        let result = traverse_step(
            Vector::from(vec![root_trav_node]),
            self,
            0,
            visited_all,
            &get_node,
            TraversalStepRecursiveResult {
                result: initial_result_vector,
                visited_all: HashSet::new(),
            },
        )?;
        let result = TraversalResult {
            step_results: result.result,
        };
        Some(result)

        // let mut rolling_result = TraversalResult::<T, E, A>::new(root_trav_node.clone());
        // let mut next_step_entries: Vector<TraversalNode<T, E, A>> = Vector::new();
        // let mut new_next_step_entries = Vector::<TraversalNode<T, E, A>>::new();
        // new_next_step_entries.push_back(root_trav_node);

        // for (current_step, step) in self.steps.iter().enumerate() {
        //     rolling_result.add_new_step();
        //     next_step_entries = new_next_step_entries.clone();
        //     new_next_step_entries = Vector::<TraversalNode<T, E, A>>::new();
        //     let step = Rc::new(step.clone());
        //     for entry_node in next_step_entries.iter() {
        //         let new_entry_node = entry_node.clone();
        //         new_entry_node.reset_step_index();
        //         let step_results = traverse_step_item(
        //             new_entry_node,
        //             step.clone(),
        //             visited_all.clone(),
        //             HashSet::<Uid>::new(),
        //             0,
        //             current_step,
        //             Vector::<TraversalNode<T, E, A>>::new(),
        //             &get_node,
        //         );

        //         if let Some(successful_branch) = step_results {
        //             rolling_result = rolling_result.add_step_result(
        //                 current_step,
        //                 successful_branch.into_traversal_step_result(),
        //             );
        //             visited_all.extend(successful_branch.nodes_visited_this_step);
        //             new_next_step_entries.extend(successful_branch.exit_nodes);
        //         } else {
        //             // If there are no successful results, then we need to roll back this exit point from the previous step
        //             let rolled_back = rolling_result
        //                 .roll_back_failed_step_item(current_step - 1, entry_node.node.id);

        //             if let Some(rolled_back) = rolled_back {
        //                 rolling_result = rolled_back;
        //             } else {
        //                 // If there is no previous step to roll back, then we need to roll back the entire traversal
        //                 return None;
        //             }
        //         }
        //     }
        // }

        // None
    }
}
