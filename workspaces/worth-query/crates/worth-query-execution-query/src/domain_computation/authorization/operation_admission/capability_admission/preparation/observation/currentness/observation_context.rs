//! Exact axes admitted for current capability observation.

use crate::domain_computation::authorization::{
    capability_registry::WorthQueryInstalledCapabilityPlan, WorthQueryRetainedCapabilityRequest,
    WorthQueryRuntimeTimeSample,
};

pub struct WorthQueryCurrentCapabilityObservation<'observation> {
    session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    installed: &'observation WorthQueryInstalledCapabilityPlan,
    request: &'observation WorthQueryRetainedCapabilityRequest,
    sample: &'observation WorthQueryRuntimeTimeSample,
}

impl WorthQueryCurrentCapabilityObservation<'_> {
    pub fn from_axes<'observation>(
        axes: super::CurrentCapabilityObservationAxes<'observation>,
    ) -> WorthQueryCurrentCapabilityObservation<'observation> {
        WorthQueryCurrentCapabilityObservation {
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
    pub const fn installed(&self) -> &WorthQueryInstalledCapabilityPlan {
        self.installed
    }
    pub const fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request
    }
    pub const fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.sample
    }
}

impl
    worth_query_execution_attempt::facade::domain_computation::authorization::delegation_admission::source_seal::SealedCapabilityObservationSource
    for WorthQueryCurrentCapabilityObservation<'_>
{
    fn session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.session_identity()
    }
    fn relational(&self) -> &worth_relational::facade::runtime::RelationalRuntime {
        self.relational()
    }
    fn snapshot(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot()
    }
    fn bridge(&self) -> &worth_runtime_bridge::facade::BridgeAuthorizationRuntime {
        self.bridge()
    }
    fn installed(
        &self,
    ) -> &crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan
    {
        self.installed()
    }
    fn request(
        &self,
    ) -> &crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest
    {
        self.request()
    }
    fn sample(&self) -> &worth_query_execution_contracts::facade::runtime_time::WorthQueryRuntimeTimeSample {
        self.sample()
    }
}
