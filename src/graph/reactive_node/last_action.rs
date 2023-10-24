use std::{io::Read, rc::Rc};

use im::{HashMap, HashSet, Vector};

use crate::prelude::{new_node::NewNode, update_node::UpdateNode, *};

use super::{read_reactive_node::ReadReactiveNode, utils::search_map_for_edge};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct LastAction<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub action_data: Rc<A>,
    pub update_info: Option<UpdateNode<T, E>>,
    pub prev_node: Option<PrevReadReactiveNode<T, E>>,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct PrevReadReactiveNode<T: GraphTraits, E: GraphTraits> {
    pub id: Uid,
    pub data: T,
    pub labels: Vector<String>,
    pub incoming_edges: HashMap<E, Vector<EdgeDescriptor<E>>>,
    pub outgoing_edges: HashMap<E, Vector<EdgeDescriptor<E>>>,
}

impl<T: GraphTraits, E: GraphTraits> PrevReadReactiveNode<T, E> {
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
            found_edges.extend(search_map_for_edge(edge_finder, &self.incoming_edges));
        }
        if !found_edges.is_empty() && edge_finder.match_all.is_none()
            || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap())
        {
            return Some(found_edges);
        }

        if search_outgoing {
            found_edges.extend(search_map_for_edge(edge_finder, &self.outgoing_edges));
        }

        if found_edges.is_empty() {
            return None;
        }

        Some(found_edges)
    }
}
