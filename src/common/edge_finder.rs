use core::fmt::Debug;
use core::hash::Hash;
use std::{rc::Rc};

use im::{hashset, HashSet};

use crate::{hooks::GetNodeClosure, prelude::reactive_node::read_reactive_node::ReadReactiveNode};

use super::{EdgeDescriptor, EdgeDir, GraphTraits, Uid};

#[derive(Clone)]
pub struct EdgeFinder<T: GraphTraits, E: GraphTraits, A: GraphTraits> {
    pub edge_type: Option<HashSet<E>>,
    pub dir: Option<EdgeDir>,
    pub host: Option<HashSet<Uid>>,
    pub target: Option<HashSet<Uid>>,
    pub render_info: Option<Option<EdgeDir>>,
    // requires a reference to the graph in order to look up the node in question and run the closure against it
    // Note that this closure is run against the target node
    pub gate_closure: Option<(
        Rc<dyn Fn(&ReadReactiveNode<T, E, A>) -> bool>,
        GetNodeClosure<T, E, A>,
    )>,
    pub match_all: Option<bool>,
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> PartialEq for EdgeFinder<T, E, A> {
    fn eq(&self, other: &Self) -> bool {
        self.edge_type == other.edge_type
            && self.dir == other.dir
            && self.host == other.host
            && self.target == other.target
            && self.render_info == other.render_info
            && self.gate_closure.is_some() == other.gate_closure.is_some()
            && self.match_all == other.match_all
    }
}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Eq for EdgeFinder<T, E, A> {}
impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Debug for EdgeFinder<T, E, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeFinder")
            .field("edge_type", &self.edge_type)
            .field("dir", &self.dir)
            .field("host", &self.host)
            .field("target", &self.target)
            .field("render_info", &self.render_info)
            .field("gate_closure", &self.gate_closure.is_some())
            .field("match_all", &self.match_all)
            .finish()
    }
}

impl<T: GraphTraits, E: GraphTraits, A: GraphTraits> Hash for EdgeFinder<T, E, A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.edge_type.hash(state);
        self.dir.hash(state);
        self.host.hash(state);
        self.target.hash(state);
        self.render_info.hash(state);
        self.gate_closure.is_some().hash(state);
        self.match_all.hash(state);
    }
}

impl<'a, T: GraphTraits, E: GraphTraits, A: GraphTraits> Default for EdgeFinder<T, E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: GraphTraits, E: GraphTraits, A: GraphTraits> EdgeFinder<T, E, A> {
    pub fn new() -> Self {
        Self {
            edge_type: None,
            dir: None,
            host: None,
            target: None,
            render_info: None,
            gate_closure: None,
            match_all: None,
        }
    }

    pub fn edge_type(&self, edge_type: E) -> Self {
        Self {
            edge_type: Some(hashset![edge_type]),
            ..self.clone()
        }
    }
    pub fn edge_types<I>(&self, edge_types: I) -> Self
    where
        I: IntoIterator<Item = E>,
    {
        Self {
            edge_type: Some(edge_types.into_iter().collect::<HashSet<E>>()),
            ..self.clone()
        }
    }

    pub fn dir(&self, direction: EdgeDir) -> Self {
        Self {
            dir: Some(direction),
            ..self.clone()
        }
    }

    pub fn host(&self, host_node: Uid) -> Self {
        Self {
            host: Some(hashset![host_node]),
            ..self.clone()
        }
    }
    pub fn hosts<I>(&self, host_nodes: I) -> Self
    where
        I: IntoIterator<Item = Uid>,
    {
        Self {
            host: Some(host_nodes.into_iter().collect::<HashSet<Uid>>()),
            ..self.clone()
        }
    }

    pub fn target(&self, target_node: Uid) -> Self {
        Self {
            target: Some(hashset![target_node]),
            ..self.clone()
        }
    }

    pub fn targets<I>(&self, target_nodes: I) -> Self
    where
        I: IntoIterator<Item = Uid>,
    {
        Self {
            target: Some(target_nodes.into_iter().collect::<HashSet<Uid>>()),
            ..self.clone()
        }
    }

    // If completely unset, it will match any render info
    // If set as None, it will match only edges with "None" render info
    // If set as Some(EdgeDir), it will match only edges with that render info
    pub fn render_info(&self, is_render: Option<EdgeDir>) -> Self {
        Self {
            render_info: Some(is_render),
            ..self.clone()
        }
    }
    pub fn match_all(&self) -> Self {
        Self {
            match_all: Some(true),
            ..self.clone()
        }
    }

    pub fn gate_closure(
        &self,
        gate_closure: impl Fn(&ReadReactiveNode<T, E, A>) -> bool + 'static,
        get_node_closure: GetNodeClosure<T, E, A>,
    ) -> Self {
        Self {
            gate_closure: Some((Rc::new(gate_closure), get_node_closure)),
            ..self.clone()
        }
    }

    pub fn invert(&self) -> Self {
        Self {
            edge_type: self.edge_type.clone(),
            dir: self.dir.clone().map(|d| d.invert()),
            host: self.target.clone(),
            target: self.host.clone(),
            render_info: self.render_info.clone().map(|ir| ir.map(|ir| ir.invert())),
            gate_closure: self.gate_closure.clone(),
            match_all: self.match_all,
        }
    }

    pub fn invert_drop_closure(&self) -> Self {
        Self {
            edge_type: self.edge_type.clone(),
            dir: self.dir.clone().map(|d| d.invert()),
            host: self.target.clone(),
            target: self.host.clone(),
            render_info: self.render_info.clone().map(|ir| ir.map(|ir| ir.invert())),
            gate_closure: None,
            match_all: self.match_all,
        }
    }

    // pub fn get_target_node_if_matches(
    //     &self,
    //     get_node_closure: GetNodeClosure<T, E, A>,
    // ) -> Option<ReadReactiveNode<T, E, A>> {
    // }

    pub fn matches(&self, edge: &EdgeDescriptor<E>) -> bool {
        let edge_type_matches = self
            .edge_type
            .as_ref()
            .map(|et| et.contains(&edge.edge_type))
            .unwrap_or(true);
        if !edge_type_matches {
            return false;
        }
        let direction_matches = self.dir.as_ref().map(|d| d == &edge.dir).unwrap_or(true);
        if !direction_matches {
            return false;
        }
        let host_node_matches = self
            .host
            .as_ref()
            .map(|hn| hn.contains(&edge.host))
            .unwrap_or(true);
        if !host_node_matches {
            return false;
        }
        let other_node_matches = self
            .target
            .as_ref()
            .map(|on| on.contains(&edge.target))
            .unwrap_or(true);
        if !other_node_matches {
            return false;
        }
        let render_info_matches = self
            .render_info
            .as_ref()
            .map(|ir| ir == &edge.render_info)
            .unwrap_or(true);
        if !render_info_matches {
            return false;
        }
        let gate_closure_matches =
            self.gate_closure
                .as_ref()
                .map_or(true, |(gate_closure, get_node)| {
                    if let Ok(node) = get_node(edge.target) {
                        gate_closure(&node)
                    } else {
                        false
                    }
                });
        if !gate_closure_matches {
            return false;
        }

        true
    }
}
