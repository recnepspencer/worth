use serde::{Deserialize, Serialize};

/// Observation of quarantine state; it cannot mutate that state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalQuarantinePosture {
    NotObserved,
    Observed,
    Unknown,
}
