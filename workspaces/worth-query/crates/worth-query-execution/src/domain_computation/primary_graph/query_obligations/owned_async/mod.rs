//! Mandatory request DATA stays distinct from opted-in process-local release.
use worth_foundational::facade::OwnedAsyncObligationIdentity;
pub use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncDeclarationDescription as QueryOwnedAsyncDefinition,
    BridgeOwnedAsyncRetention as QueryOwnedAsyncRetention,
};
pub(super) mod codec;
pub(super) mod fold;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryOwnedAsyncTerminal {
    Fulfilled,
    Rejected,
    Cancelled,
    TimedOut,
    Superseded,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryOwnedAsyncRetryCause {
    TimedOut,
    HostRequested,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryOwnedAsyncPosture {
    InFlight,
    Retryable(QueryOwnedAsyncRetryCause),
    CompletionPending,
    Terminal(QueryOwnedAsyncTerminal),
    ProcessLocalReleased(QueryOwnedAsyncTerminal),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryOwnedAsyncContinuation {
    FreshOwnerRequired,
    Terminal(QueryOwnedAsyncTerminal),
    ReleasedProcessLocal(QueryOwnedAsyncTerminal),
}
/// Native selectors are descriptive; a fresh owner issues all actual ports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryOwnedAsyncRequestGeneration {
    pub request: u64,
    pub generation: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryOwnedAsyncObligation {
    pub identity: OwnedAsyncObligationIdentity,
    pub declaration: QueryOwnedAsyncDefinition,
    pub payload: Vec<u8>,
    pub original: QueryOwnedAsyncRequestGeneration,
    pub current: QueryOwnedAsyncRequestGeneration,
    pub lineage: Vec<QueryOwnedAsyncRequestGeneration>,
    pub posture: QueryOwnedAsyncPosture,
    pub queue_depth: u64,
    pub completion: Option<Vec<u8>>,
}
impl QueryOwnedAsyncObligation {
    pub const fn continuation(&self) -> QueryOwnedAsyncContinuation {
        match self.posture {
            QueryOwnedAsyncPosture::InFlight
            | QueryOwnedAsyncPosture::Retryable(_)
            | QueryOwnedAsyncPosture::CompletionPending => {
                QueryOwnedAsyncContinuation::FreshOwnerRequired
            }
            QueryOwnedAsyncPosture::Terminal(terminal) => {
                QueryOwnedAsyncContinuation::Terminal(terminal)
            }
            QueryOwnedAsyncPosture::ProcessLocalReleased(terminal) => {
                QueryOwnedAsyncContinuation::ReleasedProcessLocal(terminal)
            }
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryOwnedAsyncRecord {
    Retain(QueryOwnedAsyncObligation),
    Progress {
        identity: OwnedAsyncObligationIdentity,
        current: QueryOwnedAsyncRequestGeneration,
        lineage: Vec<QueryOwnedAsyncRequestGeneration>,
        posture: QueryOwnedAsyncPosture,
        queue_depth: u64,
        completion: Option<Vec<u8>>,
    },
    ProcessLocalReleased {
        identity: OwnedAsyncObligationIdentity,
        terminal: QueryOwnedAsyncTerminal,
    },
    QueueDrained {
        identity: OwnedAsyncObligationIdentity,
    },
    Discharge {
        identity: OwnedAsyncObligationIdentity,
    },
}
