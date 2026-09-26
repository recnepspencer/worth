#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowInstanceState {
    Ready,
    Completed,
    /// Ended by an explicit program adoption disposition; runs no further node.
    Cancelled,
}

impl WorkflowInstanceState {
    pub(in crate::domain_computation::primary_graph) const fn persisted_tag(self) -> u64 {
        match self {
            Self::Ready => 0,
            Self::Completed => 4,
            Self::Cancelled => 5,
        }
    }
}
