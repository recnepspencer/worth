//! Retained capability decision authority and its one-way commit transition.

use std::sync::Arc;

use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::{
    WorthQueryAuthorizationDecisionFact, WorthQueryCapabilitySupportCommitBasis,
    WorthQueryPrincipalCurrentnessDependency, WorthQueryRetainedCapabilitySupport,
    WorthQueryRuntimeTimeSample,
};

pub(in crate::domain_computation) struct WorthQueryRetainedCapabilityAuthorization {
    principal: WorthQueryPrincipalCurrentnessDependency,
    decision: WorthQueryAuthorizationDecisionFact,
    capability_authority_identity: Arc<str>,
    grant: worth_relational::facade::identity::EntityId,
    request: WorthQueryRetainedCapabilityRequest,
    sample: WorthQueryRuntimeTimeSample,
    expiry: worth_foundational::facade::AspectValue,
    supporting: Option<WorthQueryRetainedCapabilitySupport>,
}

impl WorthQueryRetainedCapabilityAuthorization {
    pub(super) fn new(
        _permit: super::capability_observation::WorthQueryCapabilityRetentionPermit,
        principal: WorthQueryPrincipalCurrentnessDependency,
        decision: WorthQueryAuthorizationDecisionFact,
        capability_authority_identity: Arc<str>,
        grant: worth_relational::facade::identity::EntityId,
        request: WorthQueryRetainedCapabilityRequest,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
    ) -> Self {
        Self {
            principal,
            decision,
            capability_authority_identity,
            grant,
            request,
            sample,
            expiry,
            supporting: None,
        }
    }

    pub(super) fn retain_supporting(
        &mut self,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Result<(), ()> {
        if self.supporting.is_some()
            || supporting.decision().session_identity() != self.decision.session_identity()
        {
            return Err(());
        }
        self.supporting = Some(supporting);
        Ok(())
    }

    pub(super) const fn supporting(&self) -> Option<&WorthQueryRetainedCapabilitySupport> {
        self.supporting.as_ref()
    }

    pub(super) fn supporting_mut(&mut self) -> Option<&mut WorthQueryRetainedCapabilitySupport> {
        self.supporting.as_mut()
    }

    pub(super) fn principal(&self) -> &WorthQueryPrincipalCurrentnessDependency {
        &self.principal
    }

    pub(super) fn decision(&self) -> &WorthQueryAuthorizationDecisionFact {
        &self.decision
    }

    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.request
    }

    pub(super) const fn grant(&self) -> worth_relational::facade::identity::EntityId {
        self.grant
    }

    pub(in crate::domain_computation) const fn exact_fact_count(&self) -> usize {
        if self.supporting.is_some() {
            3
        } else {
            2
        }
    }

    pub(in crate::domain_computation) const fn authorization_requirement_count(&self) -> usize {
        if self.supporting.is_some() {
            2
        } else {
            1
        }
    }

    pub(in crate::domain_computation) fn installed_capability_identity(&self) -> [u8; 32] {
        self.request.capability_identity()
    }

    pub(in crate::domain_computation) fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.principal.session_identity() == session
            && self.decision.session_identity() == session
            && self
                .supporting
                .as_ref()
                .is_none_or(|supporting| supporting.decision().session_identity() == session)
    }

    pub(in crate::domain_computation) fn relational_counters(
        &self,
    ) -> RelationalAuthorizationObservationCounters {
        let mut counters = self.decision.relational_counters();
        if let Some(supporting) = &self.supporting {
            super::decision_facts::authorization::add_counters(
                &mut counters,
                supporting.decision().relational_counters(),
            );
        }
        counters
    }

    pub(in crate::domain_computation) fn signal_dependency_count(&self) -> usize {
        self.decision.signal_dependency_count()
            + self.supporting.as_ref().map_or(0, |supporting| {
                supporting.decision().signal_dependency_count()
            })
    }

    pub(in crate::domain_computation) fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }

    pub(in crate::domain_computation) fn decision_identity(&self) -> [u8; 32] {
        self.decision.primary_identity()
    }

    pub(super) const fn timeline(&self) -> ApplicationCapabilityValidityTimeline {
        self.sample.timeline()
    }

    pub(super) const fn sampled_value(&self) -> &worth_foundational::facade::AspectValue {
        self.sample.value()
    }

    pub(super) const fn expiry(&self) -> &worth_foundational::facade::AspectValue {
        &self.expiry
    }

    pub(super) fn bridge_is_retained(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> bool {
        self.decision.bridge_is_retained(bridge)
            && self
                .supporting
                .as_ref()
                .is_none_or(|supporting| supporting.decision().bridge_is_retained(bridge))
    }

    pub(super) fn replace_current_decision(
        &mut self,
        capability_authority_identity: &str,
        grant: worth_relational::facade::identity::EntityId,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
        decision: WorthQueryAuthorizationDecisionFact,
    ) -> Result<(), ()> {
        if self.capability_authority_identity.as_ref() != capability_authority_identity
            || self.grant != grant
            || self.sample.timeline() != sample.timeline()
            || !self.decision.has_same_lineage(&decision)
        {
            return Err(());
        }
        self.sample = sample;
        self.expiry = expiry;
        self.decision = decision;
        Ok(())
    }

    pub(super) fn replace_current_session_decision(
        &mut self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        capability_authority_identity: &str,
        grant: worth_relational::facade::identity::EntityId,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
        decision: WorthQueryAuthorizationDecisionFact,
    ) -> Result<(), ()> {
        if decision.session_identity() != session {
            return Err(());
        }
        let principal = self.principal.retained_for_session(session);
        self.replace_current_decision(
            capability_authority_identity,
            grant,
            sample,
            expiry,
            decision,
        )?;
        self.principal = principal;
        Ok(())
    }

    pub(in crate::domain_computation) fn validate_currentness_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> Result<(), super::WorthQueryOperationAuthorizationDenialKind> {
        if !self.principal.remains_current_in(runtime, snapshot) {
            return Err(super::WorthQueryOperationAuthorizationDenialKind::StalePrincipal);
        }
        if !self.decision.remains_current_in(runtime, snapshot, bridge) {
            return Err(super::WorthQueryOperationAuthorizationDenialKind::StaleAuthorization);
        }
        if self.supporting.as_ref().is_some_and(|supporting| {
            !supporting
                .decision()
                .remains_current_in(runtime, snapshot, bridge)
        }) {
            return Err(super::WorthQueryOperationAuthorizationDenialKind::StaleAuthorization);
        }
        Ok(())
    }

    pub(super) fn into_provider_commit_authorization(
        self,
        admission_identity: super::WorthQueryOperationAdmissionIdentity,
    ) -> super::WorthQueryProviderCommitAuthorization {
        let supporting = self.supporting.map(Into::into);
        let mut decisions = vec![self.decision.clone()];
        decisions.extend(supporting.as_ref().map(
            |supporting: &WorthQueryCapabilitySupportCommitBasis| supporting.decision().clone(),
        ));
        let commit = WorthQueryCapabilityCommitBasis {
            principal: self.principal.clone(),
            decision: self.decision.clone(),
            capability_authority_identity: self.capability_authority_identity,
            grant: self.grant,
            request: self.request,
            supporting,
        };
        super::WorthQueryProviderCommitAuthorization::new(
            super::WorthQueryProviderAuthorizationDecisionFacts::new(self.principal, decisions),
            super::WorthQueryCommitAuthorizationBasis::Capability {
                admission_identity,
                authorization: commit,
            },
        )
    }
}

pub(in crate::domain_computation) struct WorthQueryCapabilityCommitBasis {
    principal: WorthQueryPrincipalCurrentnessDependency,
    decision: WorthQueryAuthorizationDecisionFact,
    capability_authority_identity: Arc<str>,
    grant: worth_relational::facade::identity::EntityId,
    request: WorthQueryRetainedCapabilityRequest,
    supporting: Option<WorthQueryCapabilitySupportCommitBasis>,
}

impl WorthQueryCapabilityCommitBasis {
    pub(super) fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.principal.session_identity() == session
            && self.decision.session_identity() == session
            && self
                .supporting
                .as_ref()
                .is_none_or(|supporting| supporting.decision().session_identity() == session)
    }

    pub(super) fn principal(&self) -> &WorthQueryPrincipalCurrentnessDependency {
        &self.principal
    }

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

    pub(super) const fn supporting(&self) -> Option<&WorthQueryCapabilitySupportCommitBasis> {
        self.supporting.as_ref()
    }
}
