use core::fmt::Debug;
use core::hash::Hash;




mod edge_descriptor;
mod edge_finder;
mod graph_error;

pub use edge_descriptor::*;
pub use edge_finder::*;
pub use graph_error::*;

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

pub trait GraphTraits = Clone + PartialEq + Debug + Eq + Hash + Default + 'static;

pub type Uid = u128;
