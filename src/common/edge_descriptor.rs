use super::{EdgeDir, GraphTraits, Uid};

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
        Self {
            edge_type: self.edge_type.clone(),
            host: self.target,
            target: self.host,
            dir: self.dir.invert(),
            render_info: self
                .render_info
                .clone()
                .map(|render_info| render_info.invert()),
        }
    }
}
