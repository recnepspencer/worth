//! Preclaim the existing Bridge and Signal observation shape before lowering.

use std::mem::size_of;

use worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence;
use worth_runtime_bridge::facade::{
    BridgeAuthorizationClauseObservation, BridgeAuthorizationObservation,
    BridgeAuthorizationRequirementObservation, BridgeAuthorizationRuleDecisionEvidence,
    BridgeAuthorizationRuleObservation, BridgeAuthorizationRuntime,
};
use worth_signal::facade::{
    SignalAuthorizationClauseObservation, SignalAuthorizationRequirementObservation,
    SignalAuthorizationRuleDecisionEvidence, SignalAuthorizationRuleObservation,
};

use super::super::installed_policy::WorthQueryInstalledAuthorizationPolicy;
use super::super::WorthQueryOperationAuthorizationDenial;
use super::lower_bridge_observation;

#[derive(Debug)]
pub(in crate::domain_computation::authorization) enum BridgeObservationPreparationStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
    Observation(WorthQueryOperationAuthorizationDenial),
}

/// Installed rule and path counts are the complete allocation inventory for
/// Query lowering, Bridge lowering, and Signal evaluation. Admit all of their
/// simultaneously live vectors before the first lowering allocation. The
/// original Bridge and Signal evaluators remain the policy authorities.
pub(in crate::domain_computation::authorization) fn lower_bridge_observation_admitted<Stop>(
    installed: &WorthQueryInstalledAuthorizationPolicy,
    evidence: &RelationalAuthorizationObservationEvidence,
    dependency_identity: [u8; 32],
    policy: &str,
    bridge: &BridgeAuthorizationRuntime,
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<BridgeAuthorizationObservation, BridgeObservationPreparationStop<Stop>> {
    use BridgeObservationPreparationStop as Denial;
    let rules = installed.bridge_rule_bindings().len();
    let paths = evidence.paths().len();
    admit(
        u64::try_from(rules).map_err(|_| Denial::AccountingOverflow)?,
        0,
    )
    .map_err(Denial::Admission)?;
    let mut requirements = 0_usize;
    let mut clauses = 0_usize;
    for rule in installed.bridge_rule_bindings() {
        requirements = requirements
            .checked_add(rule.rule().requirements().len())
            .ok_or(Denial::AccountingOverflow)?;
        clauses = clauses
            .checked_add(rule.path_indices().len())
            .ok_or(Denial::AccountingOverflow)?;
    }
    let rule_bytes = size_of::<BridgeAuthorizationRuleObservation>()
        .checked_add(size_of::<SignalAuthorizationRuleObservation>())
        .and_then(|n| n.checked_add(size_of::<SignalAuthorizationRuleDecisionEvidence>()))
        .and_then(|n| n.checked_add(size_of::<BridgeAuthorizationRuleDecisionEvidence>()))
        .ok_or(Denial::AccountingOverflow)?;
    let requirement_bytes = size_of::<BridgeAuthorizationRequirementObservation>()
        .checked_add(size_of::<SignalAuthorizationRequirementObservation>())
        .ok_or(Denial::AccountingOverflow)?;
    let clause_bytes = size_of::<BridgeAuthorizationClauseObservation>()
        .checked_add(size_of::<SignalAuthorizationClauseObservation>())
        .ok_or(Denial::AccountingOverflow)?;
    let refusal_bytes = policy
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(64))
        .and_then(|n| {
            n.checked_add(size_of::<
                super::super::WorthQueryOperationAuthorizationDenialKind,
            >())
        })
        .ok_or(Denial::AccountingOverflow)?;
    let refusal_work = policy
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(65))
        .ok_or(Denial::AccountingOverflow)?;
    let backing = rules
        .checked_mul(rule_bytes)
        .and_then(|n| requirements.checked_mul(requirement_bytes)?.checked_add(n))
        .and_then(|n| clauses.checked_mul(clause_bytes)?.checked_add(n))
        .and_then(|n| n.checked_add(paths))
        .and_then(|n| n.checked_add(refusal_bytes))
        .ok_or(Denial::AccountingOverflow)?;
    // Query scans clauses once, Bridge once for shape and once for lowering,
    // and Signal once for shape, exhaustiveness and evaluation. Rule and
    // requirement visits also cover both decision-vector fills.
    let visits = rules
        .checked_mul(8)
        .and_then(|n| requirements.checked_mul(6)?.checked_add(n))
        .and_then(|n| clauses.checked_mul(6)?.checked_add(n))
        .and_then(|n| n.checked_add(paths))
        .ok_or(Denial::AccountingOverflow)?;
    let installed_count = bridge
        .correspondence_count()
        .checked_add(1)
        .ok_or(Denial::AccountingOverflow)?;
    let levels = usize::BITS as usize - installed_count.leading_zeros() as usize;
    let ordered_visits = levels.checked_mul(33).ok_or(Denial::AccountingOverflow)?;
    let work = visits
        .checked_add(ordered_visits)
        .and_then(|n| n.checked_add(refusal_work))
        .ok_or(Denial::AccountingOverflow)?;
    admit(
        u64::try_from(work).map_err(|_| Denial::AccountingOverflow)?,
        u64::try_from(backing).map_err(|_| Denial::AccountingOverflow)?,
    )
    .map_err(Denial::Admission)?;
    lower_bridge_observation(installed, evidence, dependency_identity, policy)
        .map_err(Denial::Observation)
}
