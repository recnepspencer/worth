//! Metered target-entity equality predicate matcher for related predicates.
use super::super::residual::{
    charge_work, log2_ceil, payload_bytes,
    row_operands::{observe_row_work, setup_scope_payload_bytes_for_keys},
    try_vec_exact,
};
use super::super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork};
use super::target_check::projection_unavailable;
use crate::domain_computation::primary_graph::application_query::read_execution::{
    relational_read_denial, WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_query_declaration::facade::application_query::ApplicationQueryPredicateExpected;
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
use worth_query_installation::facade::WorthQueryInstalledRelatedTargetPredicate;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{
    ProjectionAspectRequirement, ProjectionAspectScope,
    RelationalProjectedFieldObservation as Observation,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) struct TargetEqualityMatcher<'plan> {
    predicates: Vec<(
        &'plan WorthQueryInstalledRelatedTargetPredicate,
        &'plan AspectValue,
    )>,
    scope: ProjectionAspectScope,
    observe_work: u64,
}

impl<'plan> TargetEqualityMatcher<'plan> {
    pub(super) fn new<ReadIdentity>(
        read: &NarrowingRead<'_, 'plan, '_, '_, ReadIdentity>,
        targets: &'plan [WorthQueryInstalledRelatedTargetPredicate],
        target_kind: KindId,
        subject: &str,
        allocation: &SourceFactsAllocation,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        let stop = |cause| relational_read_denial(cause, subject);
        let bindings = read.plan.parameters.bindings();
        let count = targets.len();
        let lookup_work = (count as u64)
            .checked_mul(log2_ceil(bindings.len()).saturating_add(1))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        let setup_bytes = setup_scope_payload_bytes_for_keys(
            targets.iter().map(|t| (t.aspect_key(), t.field_key())),
        )
        .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        let scope_work = (count as u64)
            .checked_mul(log2_ceil(count))
            .and_then(|n| n.checked_add(setup_bytes))
            .and_then(|n| n.checked_add(lookup_work))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        charge_work(read.execution, work, scope_work)?;
        claim_setup_memory(read, count, setup_bytes, allocation, subject)?;

        let mut predicates = try_vec_exact(count)?;
        let mut keys = BTreeMap::new();
        for target in targets {
            let expected = resolve_expected(target, bindings, subject)?;
            predicates.push((target, expected));
            keys.entry(target.aspect_key().clone())
                .or_insert_with(BTreeSet::new)
                .insert(target.field_key().clone());
        }
        let scope = ProjectionAspectScope::from_requirements(
            keys.into_iter()
                .map(|(aspect, fields)| ProjectionAspectRequirement::fields(aspect, fields)),
        );
        let roster_width = read.projection.entity_aspect_binding_count(target_kind);
        let observe_work = observe_row_work(count, &scope, roster_width)
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        Ok(Self {
            predicates,
            scope,
            observe_work,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn matches<ReadIdentity>(
        &self,
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        target_id: EntityId,
        target_entity: &str,
        target_kind: KindId,
        subject: &str,
        evidence: &mut RootPathSourceBuilder,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
        let stop = |cause| relational_read_denial(cause, subject);
        for &(target, _) in &self.predicates {
            evidence.observe_guard_field(
                read.execution,
                read.projection,
                read.graph,
                target_id,
                target_entity,
                target.aspect_key(),
                target.field_key(),
                work,
            )?;
        }
        charge_work(read.execution, work, self.observe_work)?;
        let row_alloc = SourceFactsAllocation::open(read.execution).map_err(stop)?;
        let scope_clone_bytes = u64::try_from(self.scope.owned_allocation_capacity_bytes())
            .map_err(|_| stop(Cause::ChargedBytesOverflow))?;
        if scope_clone_bytes > 0 {
            row_alloc
                .claim(read.execution, scope_clone_bytes)
                .map_err(stop)?;
        }
        let mut compare_work = 0_u64;
        let matched = read
            .projection
            .entity_record_with_projection_scope(
                read.execution,
                target_id,
                self.scope.clone(),
                |record| {
                    (record.kind_id() == target_kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then(|| {
                            evaluate_record(
                                read.execution,
                                &self.predicates,
                                |a, f| record.observe_aspect_field(a, f),
                                subject,
                                &mut compare_work,
                            )
                        })
                },
            )
            .map_err(stop)?
            .ok_or_else(|| projection_unavailable(subject))??;
        if compare_work > 0 {
            work.charge_residual_filter(
                usize::try_from(compare_work).unwrap_or(usize::MAX),
                subject,
            )?;
        }
        work.charge_predicate(1, 0, self.predicates.len(), subject)?;
        Ok(matched)
    }
}

fn claim_setup_memory<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    count: usize,
    setup_bytes: u64,
    allocation: &SourceFactsAllocation,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, subject);
    allocation
        .claim_slice::<(&WorthQueryInstalledRelatedTargetPredicate, &AspectValue)>(
            read.execution,
            count,
        )
        .map_err(stop)?;
    allocation
        .claim_tree_copy::<(AspectKey, BTreeSet<FieldKey>)>(read.execution, count)
        .map_err(stop)?;
    allocation
        .claim_tree_copy::<FieldKey>(read.execution, count)
        .map_err(stop)?;
    allocation
        .claim_slice::<ProjectionAspectRequirement>(read.execution, count)
        .map_err(stop)?;
    if setup_bytes > 0 {
        allocation
            .claim(read.execution, setup_bytes)
            .map_err(stop)?;
    }
    Ok(())
}

fn resolve_expected<'plan, K: AsRef<str>>(
    target: &'plan WorthQueryInstalledRelatedTargetPredicate,
    bindings: &'plan [(K, AspectValue)],
    subject: &str,
) -> Result<&'plan AspectValue, WorthQueryApplicationReadExecutionDenial> {
    match target.expected() {
        ApplicationQueryPredicateExpected::Parameter(name) => {
            let idx = bindings
                .binary_search_by(|(param, _)| param.as_ref().cmp(name.as_str()))
                .map_err(|_| projection_unavailable(subject))?;
            Ok(&bindings[idx].1)
        }
        ApplicationQueryPredicateExpected::Value(value) => Ok(value),
    }
}

fn evaluate_record<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    predicates: &[(&WorthQueryInstalledRelatedTargetPredicate, &AspectValue)],
    observe: impl Fn(&AspectKey, &FieldKey) -> Option<Observation<'a>>,
    subject: &str,
    compare_work: &mut u64,
) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
    let stop = |cause| relational_read_denial(cause, subject);
    let mut quote_work = 0_u64;
    for &(target, expected) in predicates {
        let observed = observe(target.aspect_key(), target.field_key());
        let optional = target.presence() == ApplicationFieldPresence::Optional;
        let cmp_bytes = match observed {
            Some(Observation::Present(value)) => {
                if value.value_family() != target.scalar_family() {
                    return Err(projection_unavailable(subject));
                }
                payload_bytes(value, false).min(payload_bytes(expected, false))
            }
            Some(Observation::FieldAbsent | Observation::AspectAbsent) if optional => 0,
            _ => return Err(projection_unavailable(subject)),
        };
        quote_work = quote_work
            .checked_add(1)
            .and_then(|n| n.checked_add(cmp_bytes))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
    }
    if quote_work > 0 {
        SourceFactsAllocation::work(execution, quote_work).map_err(stop)?;
    }
    *compare_work = quote_work;
    for &(target, expected) in predicates {
        let observed = observe(target.aspect_key(), target.field_key());
        match observed {
            Some(Observation::Present(value)) if value == expected => {}
            Some(Observation::Present(_)) => return Ok(false),
            Some(Observation::FieldAbsent | Observation::AspectAbsent)
                if target.presence() == ApplicationFieldPresence::Optional =>
            {
                return Ok(false);
            }
            _ => return Err(projection_unavailable(subject)),
        }
    }
    Ok(true)
}
