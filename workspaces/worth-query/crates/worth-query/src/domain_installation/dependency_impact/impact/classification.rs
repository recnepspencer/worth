use std::collections::BTreeSet;

use worth_foundational::facade::{AspectBinding, AuthoritativeAspectChangeKind};
use worth_runtime_bridge::facade::{
    BridgeCorrespondenceDeliveryReceipt, BridgeDeliveredCorrespondenceChange,
};

use crate::domain_installation::operation_authority_chain::operation_phase_basis;
use crate::domain_installation::{
    WorthQueryConditionalOutcomeClass, WorthQueryConditionalProvenance,
};

use super::super::compiled::{
    WorthQueryCompiledSemanticAspectDependencyClosure, WorthQuerySemanticDependencyRole,
};
use super::authority::WorthQueryCheckedImpactBasis;
use super::decision_contract::{
    WorthQueryImpactAdmissionDenial, WorthQueryImpactAdmissionDenialKind, WorthQueryImpactClass,
    WorthQueryImpactCounters, WorthQueryImpactDecision,
};
use owner_accumulator::OwnerImpactAccumulator;

mod owner_accumulator;
impl WorthQueryImpactDecision {
    pub(crate) fn from_managed_live_delivery(
        closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
        delivery: &crate::ordinary::live::WorthQueryManagedLiveDelivery,
    ) -> Self {
        let owner_change_count = delivery.batches().len();
        let mut class = WorthQueryImpactClass::UnaffectedOrSuppressed;
        let mut affected_roles = BTreeSet::new();
        let mut counters = WorthQueryImpactCounters::default();
        let mut all_preclassified = true;
        for batch in delivery.batches() {
            counters.owner_changes_inspected += 1;
            let Some((mutation, preclassified)) = batch
                .mutation_delta()
                .zip(batch.preclassified_installed_impact())
            else {
                all_preclassified = false;
                continue;
            };
            if !preclassified.readmit(closure, mutation) {
                all_preclassified = false;
                continue;
            }
            class = widen_impact(class, preclassified.class());
            affected_roles.extend(preclassified.roles().iter().copied());
            let classified = preclassified.counters();
            counters.index_lookups += classified.index_lookups;
            counters.affected_edges += classified.affected_edges;
        }
        if !delivery.is_empty() && !all_preclassified {
            class = WorthQueryImpactClass::UnsupportedEscalation;
            affected_roles.clear();
            counters = WorthQueryImpactCounters {
                owner_changes_inspected: counters.owner_changes_inspected,
                ..Default::default()
            };
        }
        let affected_roles = affected_roles.into_iter().collect::<Vec<_>>();
        Self {
            class,
            affected_dependency_count: counters.affected_edges,
            affected_roles,
            owner_change_count,
            counters,
            checked_basis: WorthQueryCheckedImpactBasis::managed(closure, delivery),
        }
    }

    #[allow(dead_code)] // Phase 20 consumes this private carried authority.
    pub(crate) fn readmit_managed_delivery(
        &self,
        closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
        delivery: &crate::ordinary::live::WorthQueryManagedLiveDelivery,
    ) -> bool {
        self.checked_basis.readmit_managed(closure, delivery)
    }

    #[allow(dead_code)] // Phase 20 consumes this private carried authority.
    pub(crate) fn readmit_owner_delivery(
        &self,
        closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
        receipt: &BridgeCorrespondenceDeliveryReceipt,
        conditional: &WorthQueryConditionalProvenance,
    ) -> Result<(), WorthQueryImpactAdmissionDenial> {
        self.checked_basis
            .readmit_owner_conditional(closure, receipt, conditional)
    }
}

/// Classifies an owner-delivered Bridge change against the exact Query closure
/// and the Query-minted conditional decision produced after that delivery.
pub fn classify_owner_delivered_impact(
    closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    delivery: &BridgeCorrespondenceDeliveryReceipt,
    conditional: &WorthQueryConditionalProvenance,
) -> Result<WorthQueryImpactDecision, WorthQueryImpactAdmissionDenial> {
    let mut counters = WorthQueryImpactCounters::default();
    preflight_owner_delivery(closure, delivery, &mut counters)?;
    let change_set = delivery.change_set();
    counters.conditional_location_checks += 1;
    let location =
        crate::domain_installation::conditional_execution::query_location_from_bridge_candidate(
            change_set.dependency(),
        );
    if conditional.location() != &location {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ForeignConditionalOutcome,
            counters,
        ));
    }
    counters.conditional_authority_checks += 1;
    if operation_phase_basis(&conditional._admission) != &closure.affinity {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ConditionalAuthorityMismatch,
            counters,
        ));
    }
    counters.conditional_authority_checks += 1;
    if conditional.bridge.query_binding_identity() != closure.affinity.binding_identity {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ConditionalAuthorityMismatch,
            counters,
        ));
    }
    counters.conditional_authority_checks += 1;
    if conditional.bridge.query_capability_identity() != closure.affinity.capability_identity {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ConditionalAuthorityMismatch,
            counters,
        ));
    }
    counters.delivery_identity_checks += 1;
    if !conditional
        .bridge
        .retains_bridge_snapshot_identity(change_set.snapshot_identity())
    {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ConditionalDeliveryMismatch,
            counters,
        ));
    }

    classify_preflight_owner_delivery(closure, delivery, conditional, counters)
}

pub(crate) fn preflight_owner_delivered_truth(
    closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    truth: &worth_runtime_bridge::facade::BridgeDeliveredTruthChange,
) -> Result<(), WorthQueryImpactAdmissionDenial> {
    let mut counters = WorthQueryImpactCounters::default();
    preflight_owner_change_set(closure, truth.change_set(), &mut counters)
}

/// Revalidates semantic ownership after an explicit Query-owned binding has
/// already proved the exact primary runtime that carried the delivery.
pub(crate) fn preflight_bound_primary_truth(
    _closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    truth: &worth_runtime_bridge::facade::BridgeDeliveredTruthChange,
) -> Result<(), WorthQueryImpactAdmissionDenial> {
    let mut counters = WorthQueryImpactCounters::default();
    let change_set = truth.change_set();
    counters.delivery_identity_checks += 1;
    if change_set
        .snapshot_identity()
        .relational_snapshot_parts()
        .is_none()
    {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ConditionalDeliveryMismatch,
            counters,
        ));
    }
    Ok(())
}

fn preflight_owner_delivery(
    closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    delivery: &BridgeCorrespondenceDeliveryReceipt,
    counters: &mut WorthQueryImpactCounters,
) -> Result<(), WorthQueryImpactAdmissionDenial> {
    preflight_owner_change_set(closure, delivery.change_set(), counters)
}

fn preflight_owner_change_set(
    closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    change_set: &worth_runtime_bridge::facade::BridgeDeliveredCorrespondenceChangeSet,
    counters: &mut WorthQueryImpactCounters,
) -> Result<(), WorthQueryImpactAdmissionDenial> {
    counters.runtime_authority_checks += 1;
    if closure.affinity.installation_runtime_authority
        != change_set.basis().source_runtime_authority()
    {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ForeignRuntime,
            *counters,
        ));
    }
    counters.installation_generation_checks += 1;
    if closure.affinity.installation_generation
        != change_set.basis().source_installation_generation()
    {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::StaleInstallation,
            *counters,
        ));
    }
    counters.operation_affinity_checks += 1;
    if closure.affinity.operation_identity != change_set.basis().source_basis() {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ForeignOperation,
            *counters,
        ));
    }
    let location =
        crate::domain_installation::conditional_execution::query_location_from_bridge_candidate(
            change_set.dependency(),
        );
    counters.dependency_membership_lookups += 1;
    if !closure
        .contains_conditional_dependency(&location, change_set.dependency().dependency_ordinal())
    {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ForeignConditionalOutcome,
            *counters,
        ));
    }
    counters.delivery_identity_checks += 1;
    if change_set
        .snapshot_identity()
        .relational_snapshot_parts()
        .is_none()
    {
        return Err(impact_denial(
            WorthQueryImpactAdmissionDenialKind::ConditionalDeliveryMismatch,
            *counters,
        ));
    }
    Ok(())
}

fn classify_preflight_owner_delivery(
    closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    delivery: &BridgeCorrespondenceDeliveryReceipt,
    conditional: &WorthQueryConditionalProvenance,
    counters: WorthQueryImpactCounters,
) -> Result<WorthQueryImpactDecision, WorthQueryImpactAdmissionDenial> {
    let change_set = delivery.change_set();
    let mut accumulated = OwnerImpactAccumulator::new(change_set.changes().len(), counters);
    for change in change_set.changes() {
        accumulated.collect_change(closure, change);
    }
    accumulated.apply_conditional(
        closure,
        conditional,
        change_set.dependency().dependency_ordinal(),
    )?;
    Ok(accumulated.finish(closure, delivery, conditional))
}

const fn impact_denial(
    kind: WorthQueryImpactAdmissionDenialKind,
    counters: WorthQueryImpactCounters,
) -> WorthQueryImpactAdmissionDenial {
    WorthQueryImpactAdmissionDenial::new(kind, counters)
}

fn conditional_suppresses_output(class: WorthQueryConditionalOutcomeClass) -> bool {
    !matches!(class, WorthQueryConditionalOutcomeClass::ComputedChanged)
}

fn collect_affected_roles(
    closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    change: &BridgeDeliveredCorrespondenceChange,
    roles: &mut BTreeSet<WorthQuerySemanticDependencyRole>,
    edge_counts: &mut [usize; WorthQuerySemanticDependencyRole::COUNT],
) -> (WorthQueryImpactClass, usize) {
    let mut floor = WorthQueryImpactClass::UnaffectedOrSuppressed;
    if let Some(change) = change.semantic_change() {
        if change.kind() == AuthoritativeAspectChangeKind::Opaque {
            return (WorthQueryImpactClass::UnsupportedEscalation, 0);
        }
        let indexed = closure.indexed_semantic_impact(change);
        for role in indexed.roles {
            roles.insert(role);
            edge_counts[role.canonical_ordinal()] += 1;
        }
        let lookups = indexed.lookups;
        if matches!(change.binding(), AspectBinding::LifecycleTransition) {
            roles.insert(WorthQuerySemanticDependencyRole::SupportAndLifecycle);
            edge_counts
                [WorthQuerySemanticDependencyRole::SupportAndLifecycle.canonical_ordinal()] += 1;
            floor = match change.kind() {
                AuthoritativeAspectChangeKind::LifecycleDelete => WorthQueryImpactClass::Retirement,
                AuthoritativeAspectChangeKind::LifecycleCreate => {
                    WorthQueryImpactClass::Replacement
                }
                _ => WorthQueryImpactClass::ExplicitRebind,
            };
        }
        return (floor, lookups);
    }
    if let Some(change) = change.structural_change() {
        if matches!(
            change.kind(),
            worth_runtime_bridge::facade::BridgeCommittedRecordChangeKind::Created
                | worth_runtime_bridge::facade::BridgeCommittedRecordChangeKind::Deleted
        ) && closure.has_structural_membership_dependency()
        {
            roles.insert(WorthQuerySemanticDependencyRole::SelectionOrMembership);
            edge_counts
                [WorthQuerySemanticDependencyRole::SelectionOrMembership.canonical_ordinal()] += 1;
        }
    }
    (floor, 0)
}

fn class_for_role(role: WorthQuerySemanticDependencyRole) -> WorthQueryImpactClass {
    match role {
        WorthQuerySemanticDependencyRole::OperationalIdentity => WorthQueryImpactClass::Replacement,
        WorthQuerySemanticDependencyRole::SelectionOrMembership => {
            WorthQueryImpactClass::MembershipSplice
        }
        WorthQuerySemanticDependencyRole::Ordering | WorthQuerySemanticDependencyRole::Grouping => {
            WorthQueryImpactClass::ReorderOrRegroup
        }
        WorthQuerySemanticDependencyRole::ProjectedValue
        | WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness => {
            WorthQueryImpactClass::ValuePatch
        }
        WorthQuerySemanticDependencyRole::WindowBoundary => WorthQueryImpactClass::WindowShift,
        WorthQuerySemanticDependencyRole::SupportAndLifecycle => {
            WorthQueryImpactClass::ExplicitRebind
        }
        WorthQuerySemanticDependencyRole::InstalledDomainInvariant => {
            WorthQueryImpactClass::Reexecute
        }
        WorthQuerySemanticDependencyRole::AdvisoryOnlyContext => {
            WorthQueryImpactClass::UnaffectedOrSuppressed
        }
    }
}

fn widen_impact(
    left: WorthQueryImpactClass,
    right: WorthQueryImpactClass,
) -> WorthQueryImpactClass {
    if impact_widening_rank(left) >= impact_widening_rank(right) {
        left
    } else {
        right
    }
}

fn impact_widening_rank(class: WorthQueryImpactClass) -> u8 {
    match class {
        WorthQueryImpactClass::UnaffectedOrSuppressed => 0,
        WorthQueryImpactClass::ValuePatch => 1,
        WorthQueryImpactClass::MembershipSplice => 2,
        WorthQueryImpactClass::ReorderOrRegroup => 3,
        WorthQueryImpactClass::WindowShift => 4,
        WorthQueryImpactClass::Reexecute => 5,
        WorthQueryImpactClass::ExplicitRebind => 6,
        WorthQueryImpactClass::Replacement => 7,
        WorthQueryImpactClass::Retirement => 8,
        WorthQueryImpactClass::UnsupportedEscalation => 9,
    }
}

#[cfg(test)]
#[path = "classification_tests.rs"]
mod tests;
