//! Exact observation context for retained-authorization revalidation.

pub struct WorthQueryAuthorizationRevalidationObservation<'observation> {
    session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    installed: &'observation super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
    request: &'observation super::super::WorthQueryRetainedCapabilityRequest,
    sample: &'observation super::super::WorthQueryRuntimeTimeSample,
}

impl WorthQueryAuthorizationRevalidationObservation<'_> {
    pub fn from_axes<'observation>(
        axes: super::RevalidationObservationAxes<'observation>,
    ) -> WorthQueryAuthorizationRevalidationObservation<'observation> {
        WorthQueryAuthorizationRevalidationObservation {
            session: axes.session,
            relational: axes.relational,
            snapshot: axes.snapshot,
            bridge: axes.bridge,
            installed: axes.installed,
            request: axes.request,
            sample: axes.sample,
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
    pub const fn request(&self) -> &super::super::WorthQueryRetainedCapabilityRequest {
        self.request
    }
    pub const fn sample(&self) -> &super::super::WorthQueryRuntimeTimeSample {
        self.sample
    }
}
