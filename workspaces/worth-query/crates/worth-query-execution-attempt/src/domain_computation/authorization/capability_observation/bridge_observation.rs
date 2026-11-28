//! Runtime Bridge observation lowering from exact Relational evidence.

use worth_runtime_bridge::facade::{
    BridgeAuthorizationClauseObservation, BridgeAuthorizationObservation,
    BridgeAuthorizationRequirementObservation, BridgeAuthorizationRuleObservation,
    BridgeRecordedActorComparison,
};

use super::super::capability_registry::{
    WorthQueryCapabilityRequestGuard, WorthQueryCapabilityRequestValueAxis,
    WorthQueryInstalledCapabilityPlan,
};
use super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::super::WorthQueryOperationAuthorizationDenial;

pub(super) fn lower_bridge_observation(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    recorded: Option<&super::recorded_actor::ReadmittedActor>,
) -> Result<BridgeAuthorizationObservation, WorthQueryOperationAuthorizationDenial> {
    lower_observation(
        installed,
        projection,
        evidence,
        recorded,
        installed.correspondence(),
        installed.rules(),
        installed.paths().len(),
    )
}

pub(super) fn lower_upper_bound_observation(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    recorded: Option<&super::recorded_actor::ReadmittedActor>,
) -> Result<BridgeAuthorizationObservation, WorthQueryOperationAuthorizationDenial> {
    let upper_bound = installed
        .upper_bound()
        .as_ref()
        .ok_or_else(|| super::invalid_policy(installed.contract().name()))?;
    lower_observation(
        installed,
        projection,
        evidence,
        recorded,
        upper_bound.correspondence,
        &upper_bound.rules,
        upper_bound.path_count,
    )
}

fn lower_observation(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    recorded: Option<&super::recorded_actor::ReadmittedActor>,
    correspondence: worth_runtime_bridge::facade::BridgeAuthorizationCorrespondenceIdentity,
    rules: &[super::super::capability_registry::WorthQueryCapabilityRuleBinding],
    path_count: usize,
) -> Result<BridgeAuthorizationObservation, WorthQueryOperationAuthorizationDenial> {
    if evidence.paths().len() != path_count {
        return Err(super::invalid_policy(installed.contract().name()));
    }
    let mut observed_rules = Vec::with_capacity(rules.len());
    for rule in rules {
        let observed_requirements = if let Some((requirement, identity)) = rule.recorded_actor() {
            use worth_query_declaration::facade::application_capability::ApplicationRecordedActorComparison;
            let comparison = match requirement.comparison() {
                ApplicationRecordedActorComparison::SamePrincipal => {
                    BridgeRecordedActorComparison::SamePrincipal
                }
                ApplicationRecordedActorComparison::DifferentPrincipal => {
                    BridgeRecordedActorComparison::DifferentPrincipal
                }
            };
            let observed = recorded
                .filter(|actor| actor.native.requirement() == requirement)
                .and_then(|actor| actor.evidence.observation());
            vec![BridgeAuthorizationRequirementObservation::any([
                BridgeAuthorizationClauseObservation::from_recorded_actor(
                    identity, evidence, observed, comparison,
                ),
            ])]
        } else {
            rule.path_requirements()
                .iter()
                .map(|indices| {
                    let clauses = indices
                        .iter()
                        .map(|index| observe_clause(installed, projection, evidence, *index))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(BridgeAuthorizationRequirementObservation::any(clauses))
                })
                .collect::<Result<Vec<_>, WorthQueryOperationAuthorizationDenial>>()?
        };
        observed_rules.push(BridgeAuthorizationRuleObservation::all(
            rule.bridge().effect(),
            observed_requirements,
        ));
    }
    Ok(BridgeAuthorizationObservation::new(
        correspondence,
        evidence,
        observed_rules,
    ))
}

fn observe_clause(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    index: usize,
) -> Result<BridgeAuthorizationClauseObservation, WorthQueryOperationAuthorizationDenial> {
    let template = installed
        .paths()
        .get(index)
        .ok_or_else(|| super::invalid_policy(installed.contract().name()))?;
    let path = evidence
        .paths()
        .get(index)
        .ok_or_else(|| super::invalid_policy(installed.contract().name()))?;
    let guard = guard_matches(&template.guard, projection);
    Ok(
        BridgeAuthorizationClauseObservation::from_guarded_relational_path(
            template.identity,
            path,
            guard,
        ),
    )
}

fn guard_matches(
    guard: &WorthQueryCapabilityRequestGuard,
    projection: &WorthQueryRetainedCapabilityRequest,
) -> bool {
    let WorthQueryCapabilityRequestGuard::Accepted { axis, values } = guard else {
        return true;
    };
    let actual = match axis {
        WorthQueryCapabilityRequestValueAxis::Action => Some(projection.action()),
        WorthQueryCapabilityRequestValueAxis::Purpose => Some(projection.purpose()),
        WorthQueryCapabilityRequestValueAxis::Field => projection.field(),
        WorthQueryCapabilityRequestValueAxis::Magnitude => projection.magnitude(),
    };
    actual.is_some_and(|actual| values.contains(actual))
}
