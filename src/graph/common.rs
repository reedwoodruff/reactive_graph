use core::fmt::Debug;
use core::hash::Hash;
use std::error::Error;

use im::HashSet;

pub trait GraphTraits = Clone + PartialEq + Debug + Eq + Hash + Default + 'static;
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]

// pub type GraphError = Box<dyn Error>;

// pub struct GraphErrorStruct;
// impl Display for GraphErrorStruct {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "GraphError")
//     }
// }
// impl Error for GraphErrorStruct {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphError {
    Blueprint(String),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::Blueprint(e) => write!(f, "Blueprint Error: {}", e),
        }
    }
}
impl Error for GraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GraphError::Blueprint(_) => None,
        }
    }
}

pub type Uid = u128;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct EdgeDescriptor<E: GraphTraits> {
    pub dir: EdgeDir,
    pub edge_type: E,
    pub host: Uid,
    pub target: Uid,
    pub render_info: Option<EdgeDir>,
}

impl<E: GraphTraits> EdgeDescriptor<E> {
    pub fn new(
        host_node: Uid,
        edge_type: E,
        other_node: Uid,
        render_info: Option<EdgeDir>,
        direction: EdgeDir,
    ) -> Self {
        Self {
            host: host_node,
            dir: direction,
            edge_type,
            target: other_node,
            render_info,
        }
    }

    pub fn invert(&self) -> Self {
        let prev_host_node = self.host;
        Self {
            edge_type: self.edge_type.clone(),
            host: self.target,
            target: prev_host_node,
            dir: self.dir.invert(),
            render_info: if let Some(render_info) = self.render_info.clone() {
                Some(render_info.invert())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeDir {
    Emit,
    Recv,
}

impl EdgeDir {
    pub fn invert(&self) -> Self {
        match self {
            EdgeDir::Emit => EdgeDir::Recv,
            EdgeDir::Recv => EdgeDir::Emit,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub struct EdgeFinder<E: GraphTraits> {
    pub edge_type: Option<HashSet<E>>,
    pub dir: Option<EdgeDir>,
    pub host: Option<HashSet<Uid>>,
    pub target: Option<HashSet<Uid>>,
    pub render_info: Option<Option<EdgeDir>>,
    pub match_all: Option<bool>,
}

impl<'a, E: GraphTraits> EdgeFinder<E> {
    pub fn new() -> Self {
        Self {
            edge_type: None,
            dir: None,
            host: None,
            target: None,
            render_info: None,
            match_all: None,
        }
    }

    pub fn edge_type(&self, edge_type: HashSet<E>) -> Self {
        Self {
            edge_type: Some(edge_type),
            ..self.clone()
        }
    }

    pub fn direction(&self, direction: EdgeDir) -> Self {
        Self {
            dir: Some(direction),
            ..self.clone()
        }
    }

    pub fn host(&self, host_node: HashSet<Uid>) -> Self {
        Self {
            host: Some(host_node),
            ..self.clone()
        }
    }

    pub fn target(&self, target_node: HashSet<Uid>) -> Self {
        Self {
            target: Some(target_node),
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

    pub fn matches(&self, edge: &EdgeDescriptor<E>) -> bool {
        let edge_type_matches = self
            .edge_type
            .as_ref()
            .map(|et| et.contains(&edge.edge_type))
            .unwrap_or(true);
        let direction_matches = self.dir.as_ref().map(|d| d == &edge.dir).unwrap_or(true);
        let host_node_matches = self
            .host
            .as_ref()
            .map(|hn| hn.contains(&edge.host))
            .unwrap_or(true);
        let other_node_matches = self
            .target
            .as_ref()
            .map(|on| on.contains(&edge.target))
            .unwrap_or(true);
        let render_info_matches = self
            .render_info
            .as_ref()
            .map(|ir| ir == &edge.render_info)
            .unwrap_or(true);
        edge_type_matches
            && direction_matches
            && host_node_matches
            && other_node_matches
            && render_info_matches
    }
}
