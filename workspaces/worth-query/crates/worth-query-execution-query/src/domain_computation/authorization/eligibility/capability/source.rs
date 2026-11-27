//! The axes one fact-free capability observation reads: the same sealed
//! source admission's currentness observation supplies, over a selected
//! committed basis instead of a graph-work session's mutation basis.

use crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan;
use crate::domain_computation::authorization::delegation_admission::source_seal::SealedCapabilityObservationSource;
use crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use crate::domain_computation::authorization::WorthQueryRuntimeTimeSample;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

pub(super) struct FactFreeCapabilityObservation<'observation> {
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    pub(super) relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    pub(super) snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    pub(super) bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    pub(super) installed: &'observation WorthQueryInstalledCapabilityPlan,
    pub(super) request: &'observation WorthQueryRetainedCapabilityRequest,
    pub(super) sample: &'observation WorthQueryRuntimeTimeSample,
}

impl SealedCapabilityObservationSource for FactFreeCapabilityObservation<'_> {
    fn session_identity(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    fn relational(&self) -> &worth_relational::facade::runtime::RelationalRuntime {
        self.relational
    }
    fn snapshot(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot
    }
    fn bridge(&self) -> &worth_runtime_bridge::facade::BridgeAuthorizationRuntime {
        self.bridge
    }
    fn installed(&self) -> &WorthQueryInstalledCapabilityPlan {
        self.installed
    }
    fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request
    }
    fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.sample
    }
}
