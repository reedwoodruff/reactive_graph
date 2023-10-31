use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphError {
    Blueprint(String),
    Traversal(TraversalError),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::Blueprint(e) => write!(f, "Blueprint Error: {}", e),
            GraphError::Traversal(e) => write!(f, "Traversal Error: {}", e),
        }
    }
}
impl Error for GraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GraphError::Blueprint(_) => None,
            GraphError::Traversal(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TraversalError {
    NotFound,
    InternalError,
    TotalRollback,
}

impl std::fmt::Display for TraversalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraversalError::NotFound => write!(f, "Traversal Error: Not Found"),
            TraversalError::InternalError => write!(f, "Traversal Error: Internal Error"),
            TraversalError::TotalRollback => write!(f, "Traversal Error: Total Rollback"),
        }
    }
}
