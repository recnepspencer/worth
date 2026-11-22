pub use super::{
    decision_facts::WorthQueryObservedCommitBasis,
    delegation_admission::WorthQueryCapabilityObservationSource,
    WorthQueryApplicationCommitAuthorization, WorthQueryCapabilityCommitBasis,
    WorthQueryCommitAuthorizationBasis, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRetainedAuthorizationDecisionFacts,
};
pub struct RevalidationObservationAxes<'observation> {
    pub session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    pub snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    pub bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    pub installed: &'observation super::capability_registry::WorthQueryInstalledCapabilityPlan,
    pub request: &'observation super::WorthQueryRetainedCapabilityRequest,
    pub sample: &'observation super::WorthQueryRuntimeTimeSample,
}
pub mod currentness;
pub mod observation;
pub use observation::WorthQueryAuthorizationRevalidationObservation;
