//! Admission result transferred from the registry into actual demand progression.
use super::*;
pub enum WorthQueryOutputDemandAdvanceAdmission {
    Schedule(Option<WorthQueryRetainedOutputDemandSource>),
    Execute {
        successor_of: Option<[u8; 32]>,
    },
    AdvanceCheckpoint {
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
    },
    Ready(ReadyCompletion),
    Pending,
    Failed(WorthQueryOutputDemandDenial),
}
