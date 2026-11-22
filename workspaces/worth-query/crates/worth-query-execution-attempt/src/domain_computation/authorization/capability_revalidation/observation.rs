//! Exact admitted axes for privileged capability revalidation.

use super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;

pub struct WorthQueryCapabilityRevalidationObservation<'observation> {
    session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    installed: &'observation super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
    request: &'observation WorthQueryRetainedCapabilityRequest,
    sample: &'observation super::super::WorthQueryRuntimeTimeSample,
}

impl WorthQueryCapabilityRevalidationObservation<'_> {
    pub fn new<'observation>(
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
        installed: &'observation super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        request: &'observation WorthQueryRetainedCapabilityRequest,
        sample: &'observation super::super::WorthQueryRuntimeTimeSample,
    ) -> WorthQueryCapabilityRevalidationObservation<'observation> {
        WorthQueryCapabilityRevalidationObservation {
            session,
            relational,
            snapshot,
            bridge,
            installed,
            request,
            sample,
        }
    }

    pub const fn session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    pub const fn relational(&self) -> &worth_relational::facade::runtime::RelationalRuntime {
        self.relational
    }
    pub const fn snapshot(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot
    }
    pub const fn bridge(&self) -> &worth_runtime_bridge::facade::BridgeAuthorizationRuntime {
        self.bridge
    }
    pub const fn installed(
        &self,
    ) -> &super::super::capability_registry::WorthQueryInstalledCapabilityPlan {
        self.installed
    }
    pub const fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request
    }
    pub const fn sample(&self) -> &super::super::WorthQueryRuntimeTimeSample {
        self.sample
    }
}
