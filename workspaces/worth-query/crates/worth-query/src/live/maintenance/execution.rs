use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryAdmittedInvalidationImpact,
    WorthQueryInvalidationMaintenancePhase, WorthQueryOperationPhaseProof,
    WorthQueryOperationResultState, WorthQuerySemanticDependencyRole,
    WorthQuerySettledDomainProjection,
};

use super::{
    WorthQueryCoalescedMaintenancePlan, WorthQueryMaintenanceScope, WorthQueryMaintenanceStrategy,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMaintenanceDenial {
    ForeignOrStaleOperation,
    UnsupportedEscalation,
    PerformedEffectUnavailable,
}

pub struct WorthQueryPerformedMaintenance {
    pub(super) phase: WorthQueryOperationPhaseProof<WorthQueryInvalidationMaintenancePhase>,
    pub(super) strategies: Vec<WorthQueryMaintenanceStrategy>,
    pub(super) scope: WorthQueryMaintenanceScope,
    pub(super) roles: Vec<WorthQuerySemanticDependencyRole>,
    pub(super) result_state: WorthQueryOperationResultState,
    pub(super) execution_identity: String,
    pub(super) publication_identity: String,
    pub(super) effect: Arc<super::WorthQueryPerformedMaintenanceEffect>,
}

impl WorthQueryPerformedMaintenance {
    pub fn strategy(&self) -> WorthQueryMaintenanceStrategy {
        self.strategies[0]
    }

    pub fn strategies(&self) -> &[WorthQueryMaintenanceStrategy] {
        &self.strategies
    }

    pub const fn scope(&self) -> &WorthQueryMaintenanceScope {
        &self.scope
    }

    pub fn roles(&self) -> &[WorthQuerySemanticDependencyRole] {
        &self.roles
    }

    pub const fn result_state(&self) -> WorthQueryOperationResultState {
        self.result_state
    }

    pub fn effect(&self) -> &super::WorthQueryPerformedMaintenanceEffect {
        &self.effect
    }
}

pub(crate) fn bind_performed_invalidation_maintenance<D, O, F, L: BasisOperationLane>(
    admitted: Vec<WorthQueryAdmittedInvalidationImpact>,
    plan: &WorthQueryCoalescedMaintenancePlan,
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    performed: &crate::domain_installation::WorthQueryLiveProjectionRefresh,
    effect: Arc<super::WorthQueryPerformedMaintenanceEffect>,
) -> Result<WorthQueryPerformedMaintenance, WorthQueryMaintenanceDenial> {
    let closure = current.semantic_aspect_dependency_closure();
    let first = admitted
        .first()
        .ok_or(WorthQueryMaintenanceDenial::ForeignOrStaleOperation)?;
    if admitted.iter().any(|impact| {
        operation_phase_basis(&impact.phase) != &impact.affinity
            || impact.affinity != first.affinity
    }) || closure.invalidation_manifest().operation_identity()
        != first.affinity.operation_identity
        || closure.invalidation_manifest().installation_generation()
            != first.affinity.installation_generation
        || !performed_exactly_once(performed)
    {
        return Err(WorthQueryMaintenanceDenial::ForeignOrStaleOperation);
    }
    let parent_identity = crate::identity::hash_parts(
        &std::iter::once("worth_query_coalesced_invalidation_parent_v1".to_owned())
            .chain(
                admitted
                    .iter()
                    .map(|impact| impact.phase.payload().identity().to_owned()),
            )
            .collect::<Vec<_>>(),
    );
    Ok(performed_maintenance(
        plan,
        current,
        performed,
        effect,
        first.affinity.clone(),
        &parent_identity,
    ))
}

/// Bind maintenance that refreshed the full scope because the producer lost
/// subscription continuity. No admitted impact parents it; the current
/// closure's own authority basis does.
pub(crate) fn bind_full_scope_invalidation_maintenance<D, O, F, L: BasisOperationLane>(
    plan: &WorthQueryCoalescedMaintenancePlan,
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    performed: &crate::domain_installation::WorthQueryLiveProjectionRefresh,
    effect: Arc<super::WorthQueryPerformedMaintenanceEffect>,
) -> Result<WorthQueryPerformedMaintenance, WorthQueryMaintenanceDenial> {
    if !performed_exactly_once(performed) {
        return Err(WorthQueryMaintenanceDenial::ForeignOrStaleOperation);
    }
    let parent_identity = crate::identity::hash_parts(&[
        "worth_query_full_scope_invalidation_parent_v1".to_owned(),
        performed.authority().receipt().receipt_digest().to_owned(),
    ]);
    let affinity = current
        .semantic_aspect_dependency_closure()
        .affinity
        .clone();
    Ok(performed_maintenance(
        plan,
        current,
        performed,
        effect,
        affinity,
        &parent_identity,
    ))
}

fn performed_exactly_once(
    performed: &crate::domain_installation::WorthQueryLiveProjectionRefresh,
) -> bool {
    let work = performed.work();
    work.authority_checks() == 1
        && work.drain_calls() == 1
        && work.read_calls() == 1
        && work.projection_calls() == 1
}

fn performed_maintenance<D, O, F, L: BasisOperationLane>(
    plan: &WorthQueryCoalescedMaintenancePlan,
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    performed: &crate::domain_installation::WorthQueryLiveProjectionRefresh,
    effect: Arc<super::WorthQueryPerformedMaintenanceEffect>,
    affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
    parent_identity: &str,
) -> WorthQueryPerformedMaintenance {
    let execution_identity = performed.authority().receipt().receipt_digest().to_owned();
    let publication_identity = current.publication_receipt().identity().to_owned();
    let phase = mint_operation_phase_proof(
        format!("invalidation-maintenance:{execution_identity}"),
        Some(parent_identity),
        affinity,
    );
    WorthQueryPerformedMaintenance {
        phase,
        strategies: plan.strategies().to_vec(),
        scope: plan.scope().clone(),
        roles: plan.roles().to_vec(),
        result_state: current.result_state(),
        execution_identity,
        publication_identity,
        effect,
    }
}
