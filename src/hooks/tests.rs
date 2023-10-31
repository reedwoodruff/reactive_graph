use std::rc::Rc;

use im::Vector;
use leptos::SignalGetUntracked;

use crate::{
    prelude::*,
    traversal::{
        traversal_step::{TraversalCount},
    },
};

use super::{use_routable, use_routable_store, UseRoutableReturn};

fn setup_context() -> Rc<UseRoutableReturn<String, String, String>> {
    use_routable_store::<String, String, String>(
        None::<Vector<AllowedRenderEdgeSpecifier<String>>>,
    );
    use_routable()
}

// (1)->(2)->(3)->(4)->(5)
fn set_up_basic_graph() -> Rc<UseRoutableReturn<String, String, String>> {
    let routable = setup_context();

    let blueprint = BuildBlueprint::new();
    blueprint
        .start_with_new_node()
        .set_data("node1".to_string())
        .set_id(1)
        .set_temp_id(1)
        .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
            blue_new
                .set_temp_id(2)
                .set_id(2)
                .set_data("node2".to_string())
                .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                    blue_new
                        .set_temp_id(3)
                        .set_id(3)
                        .set_data("node3".to_string())
                        .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                            blue_new
                                .set_temp_id(4)
                                .set_id(4)
                                .set_data("node4".to_string())
                                .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
                                    blue_new
                                        .set_temp_id(5)
                                        .set_id(5)
                                        .set_data("node5".to_string())
                                })
                        })
                })
        });
    routable
        .initiate_graph(blueprint, "action_data".to_string(), 1)
        .unwrap();
    routable
}

fn add_branch_to_graph(routable: Rc<UseRoutableReturn<String, String, String>>) {
    let blueprint = BuildBlueprint::new();
    blueprint
        .start_with_new_node()
        .set_data("node6".to_string())
        .set_id(6)
        .set_temp_id(6)
        .add_edge_existing(EdgeDir::Recv, "edge_type".into(), 3, |blue_existing| {
            blue_existing
        })
        .add_edge_new(EdgeDir::Emit, "edge_type".into(), |blue_new| {
            blue_new
                .set_temp_id(7)
                .set_id(7)
                .set_data("node7".to_string())
        });
    routable
        .process_blueprint(blueprint, "action_data".to_string())
        .unwrap();
}

#[test]
fn should_setup_context() {
    let routable = setup_context();
    assert!(routable.get_node(1).is_err());
}
#[test]
fn should_allow_creating_new_graph() {
    let routable = set_up_basic_graph();
    assert!(routable.get_node(1).is_ok());
}

#[test]
fn should_allow_manipulating_existing_graph() {
    let routable = set_up_basic_graph();
    add_branch_to_graph(routable.clone());
    assert!(routable.get_node(6).is_ok());

    assert_eq!(
        routable
            .get_node(3)
            .unwrap()
            .outgoing_edges
            .get_untracked()
            .get("edge_type")
            .unwrap()
            .len(),
        2
    );
    assert!(routable.get_node(7).is_ok());
}

#[test]
fn should_return_correct_simple_traversal_results() {
    let routable = set_up_basic_graph();
    add_branch_to_graph(routable.clone());

    let traversal_descriptor = routable.traverse_search(1).add_step(
        EdgeFinder::new()
            .dir(EdgeDir::Emit)
            .edge_type("edge_type".into()),
        TraversalCount::Exactly(1),
    );

    let result = traversal_descriptor.execute();
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.step_results.len(), 1);
    assert_eq!(result.step_results[0].len(), 1);
    assert_eq!(
        result.step_results[0][0]
            .endpoints
            .iter()
            .next()
            .unwrap()
            .node
            .id,
        2
    );

    let traversal = routable
        .traverse_search(1)
        .add_step(
            EdgeFinder::new()
                .dir(EdgeDir::Emit)
                .edge_type("edge_type".into()),
            TraversalCount::Exactly(2),
        )
        .execute()
        .unwrap();

    assert_eq!(traversal.step_results.len(), 1);
    assert_eq!(traversal.step_results[0].len(), 1);
    assert_eq!(
        traversal.step_results[0][0]
            .endpoints
            .iter()
            .next()
            .unwrap()
            .node
            .id,
        3
    );
}

#[test]
fn should_return_correct_single_step_branching_traversal_results() {
    let routable: Rc<UseRoutableReturn<String, String, String>> = set_up_basic_graph();
    add_branch_to_graph(routable.clone());

    let traversal = routable
        .traverse_search(1)
        .add_step(
            EdgeFinder::new()
                .dir(EdgeDir::Emit)
                .edge_type("edge_type".into())
                .match_all(),
            TraversalCount::Exactly(3),
        )
        .execute()
        .unwrap();

    assert_eq!(traversal.step_results.len(), 1);
    assert_eq!(traversal.step_results[0].len(), 1);
    println!("{:#?}", traversal.step_results[0][0].endpoints);
    assert_eq!(traversal.step_results[0][0].endpoints.len(), 2);
    assert_eq!(
        traversal.step_results[0][0]
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.node.id == 4)
            .collect::<Vec<_>>()
            .len(),
        1
    );
    assert_eq!(
        traversal.step_results[0][0]
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.node.id == 6)
            .collect::<Vec<_>>()
            .len(),
        1
    );

    let traversal = routable
        .traverse_search(1)
        .add_step(
            EdgeFinder::new()
                .dir(EdgeDir::Emit)
                .edge_type("edge_type".into())
                .match_all(),
            TraversalCount::AtLeastExclusive(1),
        )
        .execute()
        .unwrap();

    assert!(!traversal.step_results[0][0]
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.node.id == 7)
        .collect::<Vec<_>>()
        .is_empty(),);
    assert!(!traversal.step_results[0][0]
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.node.id == 7)
        .collect::<Vec<_>>()
        .is_empty());
    println!("{:#?}", traversal.step_results[0][0].endpoints);
}

#[test]
fn should_return_correct_multi_step_branching_traversal_results() {
    let routable: Rc<UseRoutableReturn<String, String, String>> = set_up_basic_graph();
    add_branch_to_graph(routable.clone());
    let blueprint = BuildBlueprint::new();
    blueprint.start_with_update_node(4).add_edge_new(
        EdgeDir::Emit,
        "different_edge_type".into(),
        |blue_new| {
            blue_new
                .set_temp_id(8)
                .set_id(8)
                .set_data("node8".to_string())
                .add_edge_new(EdgeDir::Emit, "different_edge_type".into(), |blue_new| {
                    blue_new
                        .set_temp_id(9)
                        .set_id(9)
                        .set_data("node9".to_string())
                })
        },
    );

    blueprint.start_with_update_node(2).add_edge_new(
        EdgeDir::Emit,
        "different_edge_type".into(),
        |blue_new| {
            blue_new
                .set_temp_id(10)
                .set_id(10)
                .set_data("node10".to_string())
                .add_edge_new(EdgeDir::Emit, "different_edge_type".into(), |blue_new| {
                    blue_new
                        .set_temp_id(11)
                        .set_id(11)
                        .set_data("node11".to_string())
                })
        },
    );

    routable
        .process_blueprint(blueprint, "action_data".into())
        .unwrap();

    let traversal = routable
        .traverse_search(1)
        .add_step(
            EdgeFinder::new()
                .dir(EdgeDir::Emit)
                .edge_type("edge_type".into())
                .match_all(),
            TraversalCount::AtLeastInclusive(1),
        )
        .add_step(
            EdgeFinder::new()
                .dir(EdgeDir::Recv)
                .edge_type("edge_type".into())
                .match_all(),
            TraversalCount::Exactly(1),
        )
        .add_step(
            EdgeFinder::new()
                .dir(EdgeDir::Emit)
                .edge_type("different_edge_type".into())
                .match_all(),
            TraversalCount::BetweenExclusive(2, 3),
        )
        .execute()
        .unwrap();

    for (i, step_result) in traversal.step_results.iter().enumerate() {
        for item in step_result.iter() {
            println!(
                "step: {}, entry_point_id: {:?}, endpoints: {:?}",
                i,
                item.entry.node.id,
                item.endpoints
                    .iter()
                    .map(|item| item.node.id)
                    .collect::<Vec<_>>()
            );
        }
    }
    let first_step_results = traversal.step_results[0][0].clone();
    let first_step_endpoints = first_step_results.clone().endpoints;
    let first_step_endpoint_ids = first_step_endpoints
        .iter()
        .map(|item| item.node.id)
        .collect::<Vec<_>>();
    assert!(first_step_endpoint_ids.contains(&3));
    assert!(first_step_endpoint_ids.contains(&5));
    let node3_endpoint_result = first_step_endpoints
        .iter()
        .filter(|item| item.node.id == 3)
        .collect::<Vec<_>>();
    assert!(node3_endpoint_result.first().unwrap().step_index == 2);
    assert!(node3_endpoint_result.first().unwrap().traversal_index == 2);
    let node5_endpoint_result = first_step_endpoints
        .iter()
        .filter(|item| item.node.id == 5)
        .collect::<Vec<_>>();
    assert!(node5_endpoint_result.first().unwrap().step_index == 4);
    assert!(node5_endpoint_result.first().unwrap().traversal_index == 4);

    let final_step_11_endpoint = traversal.step_results[2]
        .iter()
        .fold(None, |acc, item| {
            item.endpoints
                .iter().find(|endpoint| endpoint.node.id == 11)
                .or(acc)
        })
        .unwrap();
    assert_eq!(final_step_11_endpoint.step_index, 2);
    assert_eq!(final_step_11_endpoint.traversal_index, 5);

    let final_step_9_endpoint = traversal.step_results[2]
        .iter()
        .fold(None, |acc, item| {
            item.endpoints
                .iter().find(|endpoint| endpoint.node.id == 9)
                .or(acc)
        })
        .unwrap();
    assert_eq!(final_step_9_endpoint.step_index, 2);
    assert_eq!(final_step_9_endpoint.traversal_index, 7);
}
