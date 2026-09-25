//! Bounded Query-owned capability delegation progression.

use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;

use super::capability_observation::WorthQueryObservedCapabilityDecision;
use super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::decision_facts::WorthQueryDelegationDecisionFact;
use super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::{
    WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRuntimeTimeSample,
};

pub(in crate::domain_computation::authorization) mod observation;
use observation::{denial, observe_lineage, observe_policy};

struct DelegationFrame {
    child_policy: WorthQueryAuthorizationDecisionFact,
    grantor: worth_relational::facade::identity::EntityId,
    parent_grant: worth_relational::facade::identity::EntityId,
    discovery: worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    transition: worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
}

pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityObservationPermit(());

impl WorthQueryCapabilityObservationPermit {
    fn new() -> Self {
        Self(())
    }
}

struct WorthQueryBoundCapabilityObservation<'observation> {
    session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation BridgeAuthorizationRuntime,
    installed: &'observation WorthQueryInstalledCapabilityPlan,
    request: &'observation WorthQueryRetainedCapabilityRequest,
    sample: &'observation WorthQueryRuntimeTimeSample,
}

mod source_seal {
    pub(in crate::domain_computation::authorization) trait SealedCapabilityObservationSource {
        fn session_identity(
            &self,
        ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
        fn relational(&self) -> &worth_relational::facade::runtime::RelationalRuntime;
        fn snapshot(&self) -> &worth_relational::facade::snapshots::SnapshotHandle;
        fn bridge(&self) -> &worth_runtime_bridge::facade::BridgeAuthorizationRuntime;
        fn installed(&self) -> &super::WorthQueryInstalledCapabilityPlan;
        fn request(&self) -> &super::WorthQueryRetainedCapabilityRequest;
        fn sample(&self) -> &super::WorthQueryRuntimeTimeSample;
    }
}

pub(in crate::domain_computation::authorization) trait WorthQueryCapabilityObservationSource:
    source_seal::SealedCapabilityObservationSource
{
    fn observe_active_capability(
        &self,
        exact_grant: Option<worth_relational::facade::identity::EntityId>,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        WorthQueryBoundCapabilityObservation::from_source(self)
            .observe_active(exact_grant, expected)
    }

    fn observe_upper_bound_capability(
        &self,
        exact_grant: worth_relational::facade::identity::EntityId,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        WorthQueryBoundCapabilityObservation::from_source(self)
            .observe_upper_bound(exact_grant, expected)
    }

    fn observe_retained_capability(
        &self,
        posture: WorthQueryCapabilityObservationPosture,
        exact_grant: worth_relational::facade::identity::EntityId,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        WorthQueryBoundCapabilityObservation::from_source(self).observe_retained(
            posture,
            exact_grant,
            expected,
        )
    }
}

impl<T> WorthQueryCapabilityObservationSource for T where
    T: source_seal::SealedCapabilityObservationSource
{
}

impl<Schema> source_seal::SealedCapabilityObservationSource
    for super::operation_progression::WorthQueryExactCapabilityObservationContext<'_, Schema>
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
    fn bridge(&self) -> &BridgeAuthorizationRuntime {
        self.bridge()
    }
    fn installed(&self) -> &WorthQueryInstalledCapabilityPlan {
        self.installed()
    }
    fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request()
    }
    fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.sample()
    }
}

impl source_seal::SealedCapabilityObservationSource
    for super::capability_revalidation::WorthQueryCapabilityRevalidationObservation<'_>
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
    fn bridge(&self) -> &BridgeAuthorizationRuntime {
        self.bridge()
    }
    fn installed(&self) -> &WorthQueryInstalledCapabilityPlan {
        self.installed()
    }
    fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request()
    }
    fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.sample()
    }
}

impl source_seal::SealedCapabilityObservationSource
    for super::authorization_revalidation::WorthQueryAuthorizationRevalidationObservation<'_>
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
    fn bridge(&self) -> &BridgeAuthorizationRuntime {
        self.bridge()
    }
    fn installed(&self) -> &WorthQueryInstalledCapabilityPlan {
        self.installed()
    }
    fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request()
    }
    fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.sample()
    }
}

impl source_seal::SealedCapabilityObservationSource
    for super::operation_progression::WorthQueryCurrentCapabilityObservation<'_>
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
    fn bridge(&self) -> &BridgeAuthorizationRuntime {
        self.bridge()
    }
    fn installed(&self) -> &WorthQueryInstalledCapabilityPlan {
        self.installed()
    }
    fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        self.request()
    }
    fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.sample()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryCapabilityObservationPosture {
    Active,
    UpperBound,
}

impl<'observation> WorthQueryBoundCapabilityObservation<'observation> {
    fn from_source(
        source: &'observation (impl source_seal::SealedCapabilityObservationSource + ?Sized),
    ) -> Self {
        Self {
            session_identity: source.session_identity(),
            relational: source.relational(),
            snapshot: source.snapshot(),
            bridge: source.bridge(),
            installed: source.installed(),
            request: source.request(),
            sample: source.sample(),
        }
    }
}

impl WorthQueryBoundCapabilityObservation<'_> {
    pub(super) fn observe_active(
        self,
        exact_grant: Option<worth_relational::facade::identity::EntityId>,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        self.observe(
            exact_grant,
            expected,
            WorthQueryCapabilityObservationPosture::Active,
        )
    }

    pub(super) fn observe_upper_bound(
        self,
        exact_grant: worth_relational::facade::identity::EntityId,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        self.observe(
            Some(exact_grant),
            expected,
            WorthQueryCapabilityObservationPosture::UpperBound,
        )
    }

    pub(super) fn observe_retained(
        self,
        posture: WorthQueryCapabilityObservationPosture,
        exact_grant: worth_relational::facade::identity::EntityId,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        self.observe(Some(exact_grant), expected, posture)
    }

    fn observe(
        self,
        exact_grant: Option<worth_relational::facade::identity::EntityId>,
        expected: Option<&WorthQueryAuthorizationDecisionFact>,
        posture: WorthQueryCapabilityObservationPosture,
    ) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
        let leaf = observe_policy(&self, posture, self.request, exact_grant)?;
        let leaf = leaf.into_seed();
        let observed = leaf.try_progress(|leaf_grant, leaf_policy| {
            observe_lineage(&self, leaf_grant, leaf_policy, posture)
        })?;
        if expected.is_some_and(|expected| !expected.has_same_lineage(observed.decision())) {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::DelegationLineageChanged,
                self.installed.contract().name(),
            ));
        }
        Ok(observed)
    }
}
