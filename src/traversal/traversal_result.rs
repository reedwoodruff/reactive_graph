use crate::prelude::*;
use im::Vector;

use super::{traversal_step_result::TraversalStepResult};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct TraversalResult<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    // pub full_result: TraversalNode<T, E, A>,
    pub step_results: Vector<Vector<TraversalStepResult<T, E, A>>>,
}

impl<'a, T: GraphTraits, E: GraphTraits, A: GraphTraits> TraversalResult<T, E, A> {
    pub fn new(num_steps: usize) -> Self {
        let mut vec = Vec::with_capacity(num_steps);
        for _ in 0..num_steps {
            vec.push(Vector::<TraversalStepResult<T, E, A>>::new());
        }
        let step_results = Vector::from(vec);
        Self {
            // full_result: start_node,
            step_results,
        }
    }

    // pub fn add_new_step(&self) -> Self {
    //     let mut new_step_results = self.step_results.clone();
    //     new_step_results.push_back(Vector::new());
    //     Self {
    //         step_results: new_step_results,
    //         ..self.clone()
    //     }
    // }

    // pub fn add_step_result(
    //     &self,
    //     step_index: usize,
    //     step_result: TraversalStepResult<T, E, A>,
    // ) -> Self {
    //     let mut new_step_results = self.step_results.clone();
    //     let mut new_step_items = new_step_results[step_index].clone();
    //     new_step_items.push_back(step_result);
    //     new_step_results.set(step_index, new_step_items);
    //     Self {
    //         step_results: new_step_results,
    //         ..self.clone()
    //     }
    // }

    // pub fn roll_back_failed_step_item(
    //     &self,
    //     step_index: usize,
    //     exit_node_id: Uid,
    // ) -> Result<Self, GraphError> {
    //     if step_index <= 0 {
    //         return Err(GraphError::Traversal(TraversalError::TotalRollback));
    //     }
    //     let mut new_step_results = self.step_results.clone();
    //     let existing_step_items = new_step_results[step_index].clone();

    //     let endpoint_index = None;
    //     let step_item_index = existing_step_items.iter().position(|step_result| {
    //         let found_endpoint = step_result
    //             .endpoints
    //             .iter()
    //             .position(|endpoint| endpoint.node.id == exit_node_id);
    //         if let Some(found_endpoint) = found_endpoint {
    //             endpoint_index = Some(found_endpoint);
    //             true
    //         } else {
    //             false
    //         }
    //     });

    //     if step_item_index.is_none() {
    //         return Err(GraphError::Traversal(TraversalError::InternalError));
    //     }
    //     if endpoint_index.is_none() {
    //         return Err(GraphError::Traversal(TraversalError::InternalError));
    //     }

    //     let endpoint_index = endpoint_index.unwrap();
    //     let new_endpoints = existing_step_items[step_item_index.unwrap()]
    //         .endpoints
    //         .clone();
    //     new_endpoints.remove(endpoint_index);

    //     // If this was the only endpoint, then we need to roll back the previous step item
    //     let new_step_items = existing_step_items.clone();
    //     if new_endpoints.len() == 0 {
    //         let removed_step_item = new_step_items.remove(step_item_index.unwrap());
    //         new_step_results.set(step_index, new_step_items);
    //         return Self {
    //             step_results: new_step_results,
    //             ..self.clone()
    //         }
    //         .roll_back_failed_step_item(step_index - 1, removed_step_item.entry.node.id);
    //     }

    //     // Edit the step item to remove the branch that failed
    // }
}
