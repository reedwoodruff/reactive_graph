use im::{HashSet, Vector};
// use leptos_reactive::{ReadSignal, SignalGetUntracked};
use leptos::*;

use crate::prelude::*;

use super::{
    super::{
        common::{EdgeDescriptor, Uid},
        GraphTraits,
    },
    last_action::LastAction,
    utils::search_map_for_edge,
};
use im::hashmap::HashMap;

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct ReadReactiveNode<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub id: Uid,
    pub data: ReadSignal<T>,
    pub labels: ReadSignal<Vector<String>>,
    pub incoming_edges: ReadSignal<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    pub outgoing_edges: ReadSignal<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    pub last_action: ReadSignal<LastAction<T, E, A>>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> ReadReactiveNode<T, E, A> {
    pub fn search_for_edge(
        &self,
        edge_finder: &EdgeFinder<E>,
    ) -> Option<HashSet<EdgeDescriptor<E>>> {
        if edge_finder.host.is_some() && !edge_finder.host.as_ref().unwrap().contains(&self.id) {
            return None;
        }

        let mut found_edges = HashSet::new();
        let search_incoming = edge_finder.dir.as_ref().is_none()
            || edge_finder.dir.as_ref().unwrap() == &EdgeDir::Recv;
        let search_outgoing = edge_finder.dir.as_ref().is_none()
            || edge_finder.dir.as_ref().unwrap() == &EdgeDir::Emit;

        if search_incoming {
            found_edges.extend(search_map_for_edge(
                edge_finder,
                &self.incoming_edges.get_untracked(),
            ));
        }
        if !found_edges.is_empty() && edge_finder.match_all.is_none()
            || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap())
        {
            return Some(found_edges);
        }

        if search_outgoing {
            found_edges.extend(search_map_for_edge(
                edge_finder,
                &self.outgoing_edges.get_untracked(),
            ));
        }

        if found_edges.is_empty() {
            return None;
        }

        Some(found_edges)
    }

    pub fn get_render_edge(&self) -> EdgeDescriptor<E> {
        self.search_for_edge(&EdgeFinder::new().render_info(Some(EdgeDir::Recv)))
            .expect("Should have render edge if node exists")
            .iter()
            .next()
            .expect("Should have render edge if node exists")
            .clone()
    }

    pub fn convert_all_edges_to_hashset(&self) -> HashSet<EdgeDescriptor<E>> {
        self.outgoing_edges
            .get_untracked()
            .iter()
            .fold(
                HashSet::new(),
                |mut acc: HashSet<EdgeDescriptor<E>>, (_edge_type, edges)| {
                    acc.extend(edges.clone());
                    acc
                },
            )
            .union(self.incoming_edges.get_untracked().iter().fold(
                HashSet::new(),
                |mut acc: HashSet<EdgeDescriptor<E>>, (_edge_type, edges)| {
                    acc.extend(edges.clone());
                    acc
                },
            ))
    }
}
