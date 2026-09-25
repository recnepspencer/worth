//! Exact governed support retained across capability lifecycle progression.

use std::sync::Arc;

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::delegation_admission::WorthQueryCapabilityObservationPosture;
use super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::{WorthQueryAuthorizationDecisionFact, WorthQueryRuntimeTimeSample};

pub(in crate::domain_computation) struct WorthQueryRetainedCapabilitySupport {
    decision: WorthQueryAuthorizationDecisionFact,
    capability_authority_identity: Arc<str>,
    grant: worth_relational::facade::identity::EntityId,
    request: WorthQueryRetainedCapabilityRequest,
    sample: WorthQueryRuntimeTimeSample,
    expiry: worth_foundational::facade::AspectValue,
    posture: WorthQueryCapabilityObservationPosture,
    role: WorthQueryCapabilitySupportRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryCapabilitySupportRole {
    DelegationTarget,
    ElevationUpperBound,
}

pub(in crate::domain_computation) struct WorthQueryCapabilitySupportCommitBasis {
    decision: WorthQueryAuthorizationDecisionFact,
    capability_authority_identity: Arc<str>,
    grant: worth_relational::facade::identity::EntityId,
    request: WorthQueryRetainedCapabilityRequest,
    posture: WorthQueryCapabilityObservationPosture,
}

impl WorthQueryRetainedCapabilitySupport {
    pub(super) fn active(
        decision: WorthQueryAuthorizationDecisionFact,
        capability_authority_identity: Arc<str>,
        grant: worth_relational::facade::identity::EntityId,
        request: WorthQueryRetainedCapabilityRequest,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
    ) -> Self {
        Self::new(
            decision,
            capability_authority_identity,
            grant,
            request,
            sample,
            expiry,
            WorthQueryCapabilityObservationPosture::Active,
            WorthQueryCapabilitySupportRole::DelegationTarget,
        )
    }

    pub(super) fn elevation_upper_bound(
        decision: WorthQueryAuthorizationDecisionFact,
        capability_authority_identity: Arc<str>,
        grant: worth_relational::facade::identity::EntityId,
        request: WorthQueryRetainedCapabilityRequest,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
    ) -> Self {
        Self::new(
            decision,
            capability_authority_identity,
            grant,
            request,
            sample,
            expiry,
            WorthQueryCapabilityObservationPosture::UpperBound,
            WorthQueryCapabilitySupportRole::ElevationUpperBound,
        )
    }

    fn new(
        decision: WorthQueryAuthorizationDecisionFact,
        capability_authority_identity: Arc<str>,
        grant: worth_relational::facade::identity::EntityId,
        request: WorthQueryRetainedCapabilityRequest,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
        posture: WorthQueryCapabilityObservationPosture,
        role: WorthQueryCapabilitySupportRole,
    ) -> Self {
        Self {
            decision,
            capability_authority_identity,
            grant,
            request,
            sample,
            expiry,
            posture,
            role,
        }
    }

    pub(super) fn replace_current_session(
        &mut self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        sample: WorthQueryRuntimeTimeSample,
        mut decision: WorthQueryAuthorizationDecisionFact,
    ) -> Result<(), ()> {
        if decision.session_identity() != session
            || self.sample.timeline() != sample.timeline()
            || !self.decision.has_same_lineage(&decision)
        {
            return Err(());
        }
        decision.retain_delegation_activation_from(&self.decision)?;
        self.sample = sample;
        self.decision = decision;
        Ok(())
    }

    pub(super) fn retained_for_operation(&self) -> Self {
        Self {
            decision: self.decision.clone(),
            capability_authority_identity: Arc::clone(&self.capability_authority_identity),
            grant: self.grant,
            request: self.request.clone(),
            sample: self.sample.clone(),
            expiry: self.expiry.clone(),
            posture: self.posture,
            role: self.role,
        }
    }

    pub(in crate::domain_computation) fn decision(&self) -> &WorthQueryAuthorizationDecisionFact {
        &self.decision
    }

    pub(super) fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }

    pub(super) const fn grant(&self) -> worth_relational::facade::identity::EntityId {
        self.grant
    }

    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.request
    }

    pub(super) const fn posture(&self) -> WorthQueryCapabilityObservationPosture {
        self.posture
    }

    pub(super) const fn role(&self) -> WorthQueryCapabilitySupportRole {
        self.role
    }

    pub(in crate::domain_computation) const fn timeline(
        &self,
    ) -> ApplicationCapabilityValidityTimeline {
        self.sample.timeline()
    }

    pub(super) const fn expiry(&self) -> &worth_foundational::facade::AspectValue {
        &self.expiry
    }
}

impl std::fmt::Debug for WorthQueryRetainedCapabilitySupport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRetainedCapabilitySupport")
            .field("capability_identity", &self.request.capability_identity())
            .field(
                "capability_authority_identity",
                &self.capability_authority_identity,
            )
            .field("grant", &self.grant)
            .finish_non_exhaustive()
    }
}

impl From<WorthQueryRetainedCapabilitySupport> for WorthQueryCapabilitySupportCommitBasis {
    fn from(supporting: WorthQueryRetainedCapabilitySupport) -> Self {
        Self {
            decision: supporting.decision,
            capability_authority_identity: supporting.capability_authority_identity,
            grant: supporting.grant,
            request: supporting.request,
            posture: supporting.posture,
        }
    }
}

impl WorthQueryCapabilitySupportCommitBasis {
    pub(super) fn decision(&self) -> &WorthQueryAuthorizationDecisionFact {
        &self.decision
    }

    pub(super) fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }

    pub(super) const fn grant(&self) -> worth_relational::facade::identity::EntityId {
        self.grant
    }

    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.request
    }

    pub(super) const fn posture(&self) -> WorthQueryCapabilityObservationPosture {
        self.posture
    }
}
