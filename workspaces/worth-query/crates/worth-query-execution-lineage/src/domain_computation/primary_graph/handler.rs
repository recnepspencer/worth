//! Pure handler value types (decision context use & interruption vocabulary).

pub mod decision_context_use;
pub use decision_context_use::DecisionContextUse;

/// Why a mutation handler stopped before finishing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerInterruption {
    /// The request was cancelled.
    Cancelled,
    /// The request reached its deadline.
    DeadlineExceeded,
}

impl std::fmt::Display for HandlerInterruption {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Cancelled => "the request was cancelled",
            Self::DeadlineExceeded => "the request reached its deadline",
        })
    }
}

impl std::error::Error for HandlerInterruption {}

impl From<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    for HandlerInterruption
{
    fn from(
        interruption: worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ) -> Self {
        match interruption {
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => Self::Cancelled,
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => Self::DeadlineExceeded,
        }
    }
}
