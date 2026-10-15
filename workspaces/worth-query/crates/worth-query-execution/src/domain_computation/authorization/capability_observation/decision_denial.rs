//! Semantic denial causes reconstructed from installed rule bindings.

use super::super::capability_registry::{
    WorthQueryCapabilityDecisionRuleBindings, WorthQueryInstalledCapabilityPlan,
};
use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use super::invalid_policy;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub(super) fn decision_denial_causes(
    bindings: &WorthQueryCapabilityDecisionRuleBindings,
    evidence: &worth_runtime_bridge::facade::BridgeAuthorizationDecisionEvidence,
    elevation: Option<WorthQueryOperationAuthorizationDenialKind>,
    recorded: Option<&crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorkflowRecordedActorObservation>,
) -> Result<Vec<WorthQueryOperationAuthorizationDenialKind>, WorthQueryOperationAuthorizationDenial>
{
    let matched = |index: usize| {
        evidence
            .rule_decisions()
            .get(index)
            .map(|decision| decision.matched())
            .ok_or_else(|| invalid_policy("capability decision rule evidence"))
    };
    let mut causes = Vec::new();
    if !matched(bindings.grant)? {
        causes.push(WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing,
        ));
    }
    append_prohibition_causes(bindings, &matched, &mut causes)?;
    if let Some(index) = bindings.recorded_actor {
        if !matched(index)? {
            causes.push(WorthQueryOperationAuthorizationDenialKind::Refused(
                match recorded {
                    None => AuthorizationRefusal::WorkflowActorContextRequired,
                    Some(recorded) if !recorded.is_present() => {
                        AuthorizationRefusal::WorkflowActorAbsent
                    }
                    Some(_) => AuthorizationRefusal::WorkflowActorComparisonMismatch,
                },
            ));
        }
    }
    if let Some(elevation) = elevation {
        causes.push(elevation);
    }
    if causes.is_empty() && !matched(bindings.allow)? {
        causes.push(WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityAuthorizationMissing,
        ));
    }
    if causes.is_empty() {
        return Err(invalid_policy(
            "denied capability without a failed semantic rule",
        ));
    }
    Ok(causes)
}

fn append_prohibition_causes(
    bindings: &WorthQueryCapabilityDecisionRuleBindings,
    matched: &impl Fn(usize) -> Result<bool, WorthQueryOperationAuthorizationDenial>,
    causes: &mut Vec<WorthQueryOperationAuthorizationDenialKind>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    for (index, kind) in [
        (
            bindings.deny,
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ExplicitDenyRuleMatched,
            ),
        ),
        (
            bindings.conflict,
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ConflictRuleMatched,
            ),
        ),
        (
            bindings.separation_of_duty,
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::SeparationOfDutyRuleMatched,
            ),
        ),
        (
            bindings.distinct_actor,
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DistinctActorRuleMatched,
            ),
        ),
    ] {
        if let Some(index) = index {
            if matched(index)? {
                causes.push(kind);
            }
        }
    }
    Ok(())
}

pub(super) fn elevation_denial_kind(
    installed: &WorthQueryInstalledCapabilityPlan,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
) -> Option<WorthQueryOperationAuthorizationDenialKind> {
    let elevation = installed.elevation().as_ref()?;
    let active = evidence.paths().get(elevation.active_path_index)?;
    let not_before = evidence
        .paths()
        .get(elevation.temporal.not_before_path_index)?;
    let not_after = evidence
        .paths()
        .get(elevation.temporal.not_after_path_index)?;
    let expired = evidence.paths().get(elevation.expired_path_index)?;
    let self_approval = evidence.paths().get(elevation.self_approval_path_index)?;
    if self_approval.matched() {
        Some(WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationSelfApproval,
        ))
    } else if requirements_match(evidence, &elevation.approver_conflict_requirements) {
        Some(WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationApproverConflict,
        ))
    } else if expired.matched()
        || (active.matched() && not_before.matched() && !not_after.matched())
    {
        Some(WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationExpired,
        ))
    } else if !active.matched() || !not_before.matched() {
        Some(WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationInactive,
        ))
    } else {
        None
    }
}

fn requirements_match(
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    requirements: &[Vec<usize>],
) -> bool {
    requirements.iter().all(|requirement| {
        requirement.iter().any(|path_index| {
            evidence
                .paths()
                .get(*path_index)
                .is_some_and(|path| path.matched())
        })
    })
}
