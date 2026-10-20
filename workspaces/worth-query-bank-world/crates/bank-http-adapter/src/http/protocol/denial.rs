use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpDenialKind {
    MalformedRequest,
    ProducerSourceSizeOverflow,
    ProducerSourceNotEncodable,
    UnsupportedProtocol,
    Unauthenticated,
    PermissionDenied,
    NotFound,
    Cancelled,
    DeadlineExceeded,
    Stale,
    Unavailable,
    ResourceExhausted,
    Saturated,
    InternalDenied,
    ManagedCleanupRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpNextAction {
    None,
    Authenticate,
    CorrectRequest,
    Retry,
    Refresh,
    NarrowRequest,
    ContactOperator,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankHttpDenial {
    pub kind: BankHttpDenialKind,
    pub next_action: BankHttpNextAction,
}

impl BankHttpDenial {
    pub const fn new(kind: BankHttpDenialKind, next_action: BankHttpNextAction) -> Self {
        Self { kind, next_action }
    }
}
