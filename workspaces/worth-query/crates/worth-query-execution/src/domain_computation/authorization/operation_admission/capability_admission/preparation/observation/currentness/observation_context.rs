//! Exact axes admitted for current capability observation.

use crate::domain_computation::authorization::{
    capability_registry::WorthQueryInstalledCapabilityPlan, WorthQueryRetainedCapabilityRequest,
    WorthQueryRuntimeTimeSample,
};

pub(in crate::domain_computation::authorization) struct WorthQueryCurrentCapabilityObservation<
    'observation,
> {
    session: crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    installed: &'observation WorthQueryInstalledCapabilityPlan,
    request: &'observation WorthQueryRetainedCapabilityRequest,
    sample: &'observation WorthQueryRuntimeTimeSample,
}

impl WorthQueryCurrentCapabilityObservation<'_> {
    pub(super) fn from_axes<'observation>(
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

    pub(in crate::domain_computation::authorization) const fn session_identity(
        &self,
    ) -> crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity{
        self.session
    }
    pub(in crate::domain_computation::authorization) const fn relational(
        &self,
    ) -> &worth_relational::facade::runtime::RelationalRuntime {
        self.relational
    }
    pub(in crate::domain_computation::authorization) const fn snapshot(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot
    }
    pub(in crate::domain_computation::authorization) const fn bridge(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeAuthorizationRuntime {
        self.bridge
    }
    pub(in crate::domain_computation::authorization) const fn installed(
        &self,
    ) -> &WorthQueryInstalledCapabilityPlan {
        self.installed
    }
    pub(in crate::domain_computation::authorization) const fn request(
        &self,
    ) -> &WorthQueryRetainedCapabilityRequest {
        self.request
    }
    pub(in crate::domain_computation::authorization) const fn sample(
        &self,
    ) -> &WorthQueryRuntimeTimeSample {
        self.sample
    }
}
