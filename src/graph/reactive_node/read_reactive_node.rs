use im::{HashSet, Vector};
// use leptos_reactive::{ReadSignal, SignalGetUntracked};
use leptos::*;

use crate::prelude::*;

use super::super::{
    common::{EdgeDescriptor, Uid},
    GraphTraits,
};
use im::hashmap::HashMap;

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct ReadReactiveNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub data: ReadSignal<T>,
    pub labels: ReadSignal<Vector<String>>,
    pub incoming_edges: ReadSignal<HashMap<E, Vector<EdgeDescriptor<E>>>>,
    pub outgoing_edges: ReadSignal<HashMap<E, Vector<EdgeDescriptor<E>>>>,
}

impl<T: GraphTraits, E: GraphTraits> ReadReactiveNode<T, E> {
    fn search_map_for_edge<I>(
        edge_finder: &EdgeFinder<E>,
        map: &HashMap<E, I>,
    ) -> HashSet<EdgeDescriptor<E>>
    where
        I: IntoIterator<Item = EdgeDescriptor<E>> + Clone,
    {
        let mut found_edges = HashSet::new();
        for (edge_type, edges) in map.iter() {
            // Return the array of one if the edgefinder is not set to match all
            if !found_edges.is_empty()
                && (edge_finder.match_all.is_none()
                    || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap()))
            {
                break;
            }

            if edge_finder.edge_type.as_ref().is_some()
                && !edge_finder.edge_type.as_ref().unwrap().contains(edge_type)
            {
                continue;
            }
            for edge in edges.clone().into_iter() {
                // Return the array of one if the edgefinder is not set to match all
                if !found_edges.is_empty()
                    && (edge_finder.match_all.is_none()
                        || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap()))
                {
                    break;
                }

                if edge_finder.dir.as_ref().is_some()
                    && edge_finder.dir.as_ref().unwrap() != &edge.dir
                {
                    panic!("Edge direction does not match");
                }

                if edge_finder.host.is_some()
                    && !edge_finder.host.as_ref().unwrap().contains(&edge.host)
                {
                    continue;
                }
                if edge_finder.target.is_some()
                    && !edge_finder.target.as_ref().unwrap().contains(&edge.target)
                {
                    continue;
                }
                if edge_finder.render_info.is_some()
                    && edge_finder.render_info.as_ref().unwrap() != &edge.render_info
                {
                    continue;
                }
                found_edges.insert(edge.clone());
            }
        }
        found_edges
    }

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
            found_edges.extend(ReadReactiveNode::<T, E>::search_map_for_edge(
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
            found_edges.extend(ReadReactiveNode::<T, E>::search_map_for_edge(
                edge_finder,
                &self.outgoing_edges.get_untracked(),
            ));
        }

        if found_edges.is_empty() {
            return None;
        }

        Some(found_edges)
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
            .union(self.outgoing_edges.get_untracked().iter().fold(
                HashSet::new(),
                |mut acc: HashSet<EdgeDescriptor<E>>, (_edge_type, edges)| {
                    acc.extend(edges.clone());
                    acc
                },
            ))
    }
}
