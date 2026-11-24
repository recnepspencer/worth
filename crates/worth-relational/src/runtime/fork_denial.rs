//! Stable refusal for a whole-runtime fork.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRuntimeForkDenial {
    PublicationInFlight,
    PerformedPublicationRequiresSettlement {
        commit_id: crate::history::data::coordinates::CommitId,
    },
    IdentityCapacityExhausted,
    CanonicalInventoryInvalid,
    BranchIdentityRebind(
        crate::branch::reference::observation::RelationalBranchObservationConstructionDenial,
    ),
}
