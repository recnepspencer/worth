#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowInstanceState {
    Ready,
    Completed,
    /// Ended by an explicit cancellation or a program adoption disposition;
    /// runs no further node. Effects it performed remain.
    Cancelled,
    /// Ended by an explicit migration; its successor continues the work.
    Migrated,
    /// Ended by an authored business action; its source stage and history remain.
    Terminated,
}

impl WorkflowInstanceState {
    pub(in crate::domain_computation::primary_graph) const fn persisted_tag(self) -> u64 {
        match self {
            Self::Ready => 0,
            Self::Completed => 4,
            Self::Cancelled => 5,
            Self::Migrated => 6,
            Self::Terminated => 7,
        }
    }
}
