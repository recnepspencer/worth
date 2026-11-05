//! Moved whole from `domain_computation::primary_graph::application_runtime::execution_memory`: the items below and every impl they own.
use worth_execution::LeaseDenial;

/// Refusal to admit explicit execution resources through this application's World.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionLeaseDenial {
    /// The host installed no process execution authority in this World.
    Unavailable,
    /// The installed execution authority refused the caller's lease request.
    Lease(LeaseDenial),
}

impl std::fmt::Display for WorthQueryExecutionLeaseDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("World has no installed execution authority"),
            Self::Lease(denial) => write!(formatter, "World execution lease refused: {denial:?}"),
        }
    }
}

impl std::error::Error for WorthQueryExecutionLeaseDenial {}
