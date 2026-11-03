//! Admission result transferred from the registry into actual demand progression.
use super::*;
pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputDemandAdvanceAdmission {
    Schedule(Option<WorthQueryPerformedOutputDemandSource>),
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
