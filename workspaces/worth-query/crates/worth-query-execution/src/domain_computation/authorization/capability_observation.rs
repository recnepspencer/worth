//! Capability observation across Relational and Runtime Bridge authority.

use worth_relational::facade::authorization::RelationalAuthorizationObservationPlan;
use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;

use super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::{
    WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::authorization::WorthQueryRuntimeTimeSample;

mod bridge_observation;
mod decision_denial;
mod elevation;
mod expiry;
mod grant_selection;
mod path_preparation;
mod projection_validation;
mod upper_bound;
pub(super) use upper_bound::observe_upper_bound_policy;

pub(super) struct WorthQueryObservedCapabilityDecision {
    decision: WorthQueryAuthorizationDecisionFact,
    grant: worth_relational::facade::identity::EntityId,
    capability_authority_identity: std::sync::Arc<str>,
    request: WorthQueryRetainedCapabilityRequest,
    sample: WorthQueryRuntimeTimeSample,
    expiry: worth_foundational::facade::AspectValue,
}

pub(super) struct WorthQueryObservedCapabilitySeed {
    decision: WorthQueryAuthorizationDecisionFact,
    grant: worth_relational::facade::identity::EntityId,
    capability_authority_identity: std::sync::Arc<str>,
    request: WorthQueryRetainedCapabilityRequest,
    sample: WorthQueryRuntimeTimeSample,
    expiry: worth_foundational::facade::AspectValue,
}

pub(in crate::domain_computation::authorization) struct WorthQueryAuthorizationDecisionPermit(());
pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityRetentionPermit(());

impl WorthQueryAuthorizationDecisionPermit {
    fn new() -> Self {
        Self(())
    }
}

impl WorthQueryCapabilityRetentionPermit {
    fn new() -> Self {
        Self(())
    }
}

impl WorthQueryObservedCapabilityDecision {
    const fn new(
        decision: WorthQueryAuthorizationDecisionFact,
        grant: worth_relational::facade::identity::EntityId,
        capability_authority_identity: std::sync::Arc<str>,
        request: WorthQueryRetainedCapabilityRequest,
        sample: WorthQueryRuntimeTimeSample,
        expiry: worth_foundational::facade::AspectValue,
    ) -> Self {
        Self {
            decision,
            grant,
            capability_authority_identity,
            request,
            sample,
            expiry,
        }
    }

    pub(super) const fn grant(&self) -> worth_relational::facade::identity::EntityId {
        self.grant
    }
    pub(super) const fn decision(&self) -> &WorthQueryAuthorizationDecisionFact {
        &self.decision
    }

    pub(super) fn into_refresh_for_grant(
        self,
        expected: worth_relational::facade::identity::EntityId,
    ) -> Result<
        (
            WorthQueryAuthorizationDecisionFact,
            worth_foundational::facade::AspectValue,
        ),
        (),
    > {
        (self.grant == expected)
            .then_some((self.decision, self.expiry))
            .ok_or(())
    }

    pub(super) fn into_decision_for_grant(
        self,
        expected: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryAuthorizationDecisionFact, ()> {
        (self.grant == expected).then_some(self.decision).ok_or(())
    }

    pub(super) fn into_seed(self) -> WorthQueryObservedCapabilitySeed {
        WorthQueryObservedCapabilitySeed {
            decision: self.decision,
            grant: self.grant,
            capability_authority_identity: self.capability_authority_identity,
            request: self.request,
            sample: self.sample,
            expiry: self.expiry,
        }
    }

    pub(super) fn into_retained_authorization(
        self,
        principal: super::WorthQueryPrincipalCurrentnessDependency,
    ) -> super::WorthQueryRetainedCapabilityAuthorization {
        super::WorthQueryRetainedCapabilityAuthorization::new(
            WorthQueryCapabilityRetentionPermit::new(),
            principal,
            self.decision,
            self.capability_authority_identity,
            self.grant,
            self.request,
            self.sample,
            self.expiry,
        )
    }
}

impl WorthQueryObservedCapabilitySeed {
    pub(super) fn try_progress<E>(
        self,
        transition: impl FnOnce(
            worth_relational::facade::identity::EntityId,
            WorthQueryAuthorizationDecisionFact,
        ) -> Result<WorthQueryAuthorizationDecisionFact, E>,
    ) -> Result<WorthQueryObservedCapabilityDecision, E> {
        transition(self.grant, self.decision).map(|decision| {
            WorthQueryObservedCapabilityDecision::new(
                decision,
                self.grant,
                self.capability_authority_identity,
                self.request,
                self.sample,
                self.expiry,
            )
        })
    }
}

pub(super) fn observe_capability_policy(
    _permit: super::delegation_admission::WorthQueryCapabilityObservationPermit,
    session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &BridgeAuthorizationRuntime,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    exact_grant: Option<worth_relational::facade::identity::EntityId>,
) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
    projection_validation::validate_projection_shape(
        installed,
        request,
        installed.paths().len(),
        installed.elevation().is_some(),
    )?;
    let (exact_grant, mut preparatory_relational_work) = resolve_exact_grant(
        relational,
        snapshot.clone(),
        installed,
        request,
        sample,
        exact_grant,
    )?;
    let paths =
        path_preparation::prepare_exact_policy_paths(installed, request, sample, exact_grant)?;
    let evidence = observe_exact_policy(relational, snapshot.clone(), installed, request, paths)?;
    let bridge_evidence = evaluate_exact_policy(bridge, installed, request, &evidence)?;
    let grant = extract_exact_grant(installed, &evidence)?;
    if exact_grant != grant {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            installed.contract().name(),
        ));
    }
    let expiry = expiry::observe_grant_expiry(relational, &snapshot, installed, grant)?;
    expiry::add_observation_work(&mut preparatory_relational_work);
    Ok(WorthQueryObservedCapabilityDecision {
        decision: WorthQueryAuthorizationDecisionFact::from_capability_observation(
            WorthQueryAuthorizationDecisionPermit::new(),
            session_identity,
            evidence,
            bridge_evidence,
        )
        .with_preparatory_relational_work(preparatory_relational_work),
        grant,
        capability_authority_identity: std::sync::Arc::clone(
            installed.capability_authority_identity(),
        ),
        request: request.clone(),
        sample: sample.clone(),
        expiry,
    })
}

fn resolve_exact_grant(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    exact_grant: Option<worth_relational::facade::identity::EntityId>,
) -> Result<
    (
        worth_relational::facade::identity::EntityId,
        worth_relational::facade::authorization::RelationalAuthorizationObservationCounters,
    ),
    WorthQueryOperationAuthorizationDenial,
> {
    match exact_grant {
        Some(grant) => Ok((grant, Default::default())),
        None => {
            grant_selection::select_exact_grant(relational, snapshot, installed, request, sample)
                .map(grant_selection::SelectedCapabilityGrant::into_parts)
        }
    }
}

fn observe_exact_policy(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    paths: Vec<worth_relational::facade::authorization::RelationalAuthorizationPathPlan>,
) -> Result<
    worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    WorthQueryOperationAuthorizationDenial,
> {
    let observation_plan = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        request.principal(),
        request.resource(),
        installed.principal_kind(),
        installed.scope_kind(),
        paths,
        [],
    )
    .map_err(|_| invalid_policy(installed.contract().name()))?;
    relational
        .observe_authorization(observation_plan)
        .map_err(|_| {
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected,
                installed.contract().name(),
            )
        })
}

fn evaluate_exact_policy(
    bridge: &BridgeAuthorizationRuntime,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
) -> Result<
    worth_runtime_bridge::facade::BridgeAuthorizationDecisionEvidence,
    WorthQueryOperationAuthorizationDenial,
> {
    let dependency_identity = *evidence.observation_identity().bytes();
    let bridge_observation = bridge_observation::lower_bridge_observation(
        installed,
        request,
        evidence,
        dependency_identity,
    )?;
    let bridge_evidence = bridge.evaluate(bridge_observation).map_err(|_| {
        WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::BridgeEvaluationRejected,
            installed.contract().name(),
        )
    })?;
    if bridge_evidence.dependency_identity() != &dependency_identity
        || !bridge.retains(&bridge_evidence)
    {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            installed.contract().name(),
        ));
    }
    if !bridge_evidence.is_allowed() {
        let causes = decision_denial::decision_denial_causes(
            installed.decision_rules(),
            &bridge_evidence,
            decision_denial::elevation_denial_kind(installed, evidence),
        )?;
        return Err(WorthQueryOperationAuthorizationDenial::from_ordered_causes(
            causes,
            installed.contract().name(),
        ));
    }
    Ok(bridge_evidence)
}

fn extract_exact_grant(
    installed: &WorthQueryInstalledCapabilityPlan,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
) -> Result<worth_relational::facade::identity::EntityId, WorthQueryOperationAuthorizationDenial> {
    evidence
        .paths()
        .get(installed.grant_witness().path_index())
        .and_then(|path| path.witness())
        .and_then(|witness| witness.entity_at(installed.grant_witness().entity_ordinal()))
        .ok_or_else(|| invalid_policy(installed.contract().name()))
}

pub(super) fn invalid_policy(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::InvalidInstalledPolicy,
        subject,
    )
}
