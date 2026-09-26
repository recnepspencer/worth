//! Delegation-lineage and policy observation for one bound capability context.

use std::collections::BTreeSet;

use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRule;

use super::{
    DelegationFrame, WorthQueryBoundCapabilityObservation, WorthQueryCapabilityObservationPermit,
    WorthQueryCapabilityObservationPosture,
};

mod discovery;
mod transition;

struct ObservedDelegationParent {
    evidence: worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    parent: Option<worth_relational::facade::identity::EntityId>,
}

pub(in crate::domain_computation::authorization) struct ObservedDelegationTransition {
    evidence: worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    grantor: worth_relational::facade::identity::EntityId,
}

impl ObservedDelegationTransition {
    pub(in crate::domain_computation::authorization) const fn grantor(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.grantor
    }
}
use super::{WorthQueryDelegationDecisionFact, WorthQueryObservedCapabilityDecision};
use crate::domain_computation::authorization::{
    WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRetainedCapabilityRequest,
};

pub(super) fn observe_lineage(
    observation: &WorthQueryBoundCapabilityObservation<'_>,
    leaf_grant: worth_relational::facade::identity::EntityId,
    leaf_policy: WorthQueryAuthorizationDecisionFact,
    posture: WorthQueryCapabilityObservationPosture,
) -> Result<WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial> {
    let installed = observation.installed;
    let request = observation.request;
    let mut visited = BTreeSet::from([leaf_grant]);
    let mut frames: Vec<DelegationFrame> = Vec::new();
    let mut current_grant = leaf_grant;
    let mut current_principal = request.principal();
    let mut current_policy = leaf_policy;
    loop {
        let status_revision = observation
            .relational
            .read_truth()
            .project_snapshot(observation.snapshot)
            .and_then(|view| {
                view.entity_field_revision(
                    current_grant,
                    &worth_foundational::facade::AspectFieldLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        installed
                            .delegation()
                            .active_status
                            .0
                            .aspect()
                            .aspect_key()
                            .clone(),
                        installed.delegation().active_status.0.field_path().clone(),
                    ),
                )
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    installed.contract().name(),
                )
            })?;
        let observed = discovery::observe_parent(
            observation.relational,
            observation.snapshot.clone(),
            installed,
            current_grant,
            current_principal,
        )?;
        let Some(parent_grant) = observed.parent else {
            let mut decision = current_policy.with_delegation(
                WorthQueryDelegationDecisionFact::root(
                    observation.relational,
                    observed.evidence,
                    status_revision,
                )
                .map_err(|()| {
                    denial(
                        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                        installed.contract().name(),
                    )
                })?,
            );
            while let Some(frame) = frames.pop() {
                decision = frame.child_policy.with_delegation(
                    WorthQueryDelegationDecisionFact::delegated(
                        observation.relational,
                        frame.grantor,
                        frame.parent_grant,
                        frame.discovery,
                        frame.transition,
                        decision,
                        frame.status_revision,
                    )
                    .map_err(|()| {
                        denial(
                            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                            installed.contract().name(),
                        )
                    })?,
                );
            }
            return Ok(decision);
        };
        if !visited.insert(parent_grant) {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::DelegationCycle,
                installed.contract().name(),
            ));
        }
        enforce_depth(
            installed.delegation().rule,
            frames.len() + 1,
            installed.contract().name(),
        )?;
        let transition = transition::observe_transition(
            observation.relational,
            observation.snapshot.clone(),
            installed,
            request,
            observation.sample,
            current_grant,
            parent_grant,
        )?;
        let parent_request = request.for_delegation_parent(&transition);
        let parent = observe_policy(observation, posture, &parent_request, Some(parent_grant))?;
        let parent_policy = parent.into_decision_for_grant(parent_grant).map_err(|()| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::DelegationRejected,
                installed.contract().name(),
            )
        })?;
        frames.push(DelegationFrame {
            child_policy: current_policy,
            grantor: transition.grantor,
            parent_grant,
            discovery: observed.evidence,
            transition: transition.evidence,
            status_revision,
        });
        current_grant = parent_grant;
        current_principal = transition.grantor;
        current_policy = parent_policy;
    }
}

pub(super) fn observe_policy(
    observation: &WorthQueryBoundCapabilityObservation<'_>,
    posture: WorthQueryCapabilityObservationPosture,
    request: &WorthQueryRetainedCapabilityRequest,
    exact_grant: Option<worth_relational::facade::identity::EntityId>,
) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
    let installed = observation.installed;
    match posture {
        WorthQueryCapabilityObservationPosture::Active => {
            super::super::capability_observation::observe_capability_policy(
                WorthQueryCapabilityObservationPermit::new(),
                observation.session_identity,
                observation.relational,
                observation.snapshot.clone(),
                observation.bridge,
                installed,
                request,
                observation.sample,
                exact_grant,
            )
        }
        WorthQueryCapabilityObservationPosture::UpperBound => {
            super::super::capability_observation::observe_upper_bound_policy(
                WorthQueryCapabilityObservationPermit::new(),
                observation.session_identity,
                observation.relational,
                observation.snapshot.clone(),
                observation.bridge,
                installed,
                request,
                observation.sample,
                exact_grant.ok_or_else(|| {
                    denial(
                        WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected,
                        installed.contract().name(),
                    )
                })?,
            )
        }
    }
}

fn enforce_depth(
    rule: ApplicationCapabilityDelegationRule,
    depth: usize,
    subject: &str,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let maximum = match rule {
        ApplicationCapabilityDelegationRule::Forbidden => {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::DelegationRejected,
                subject,
            ));
        }
        ApplicationCapabilityDelegationRule::NarrowAllDimensions { maximum_depth } => {
            maximum_depth.maximum() as usize
        }
    };
    if depth > maximum {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::DelegationDepthExceeded,
            subject,
        ));
    }
    Ok(())
}

pub(super) fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
