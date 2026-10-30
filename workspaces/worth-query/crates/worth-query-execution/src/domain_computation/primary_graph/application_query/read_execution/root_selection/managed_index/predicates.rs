//! Apply the existing governed frontier predicates to actual provider candidates.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn filter(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &ReadPlan<'_>,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    mut frontier: BTreeSet<EntityId>,
    source: &mut RootPathSourceBuilder,
    work: &mut RootSelectionWork<'_>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<BTreeSet<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let storage = MaterialCompletion::new(execution, allocation);
    let stop = |cause| relational_read_denial(cause, plan.name);
    storage
        .work(
            u64::try_from(plan.contract.predicates().len())
                .map_err(|_| stop(Cause::WorkCounterOverflow))?,
        )
        .map_err(stop)?;
    for predicate in plan.contract.predicates() {
        let (entity, aspect, field) = predicate.field();
        let names = entity
            .len()
            .checked_add(aspect.len())
            .and_then(|n| n.checked_add(field.len()))
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        allocation
            .claim_slice::<u8>(execution, names)
            .map_err(stop)?;
        // equality_field allocates these three lookup keys. Its map upper
        // bound is available without visiting any unrelated field entries.
        let count = graph
            .equality_index_ids()
            .size_hint()
            .1
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        let units = names
            .checked_add(1)
            .and_then(|n| n.checked_mul(count.checked_add(1)?))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        storage.work(units).map_err(stop)?;
        let admitted = plan
            .governance
            .admit_internal_projection(
                predicate.field(),
                predicate.field_key(),
                ApplicationQueryObservableInfluence::RowPresence,
            )
            .ok_or_else(|| plan.predicate_unavailable_denial())?;
        let layout = graph
            .equality_field(entity, aspect, field)
            .ok_or_else(|| plan.predicate_unavailable_denial())?;
        if !admitted.admits_locator(&layout.locator) {
            return Err(plan.predicate_unavailable_denial());
        }
        let mut expected = None;
        for (name, value) in plan.parameters.bindings() {
            let units = name
                .len()
                .checked_add(predicate.parameter().len())
                .and_then(|n| n.checked_add(1))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
            storage.work(units).map_err(stop)?;
            if *name == predicate.parameter() {
                expected = Some(value);
                break;
            }
        }
        let expected = expected.ok_or_else(|| plan.predicate_unavailable_denial())?;
        for root in &frontier {
            source.observe_guard_field(
                execution,
                projection,
                graph,
                *root,
                entity,
                predicate.aspect_key(),
                predicate.field_key(),
                work,
            )?;
        }
        let result = path_union::read_governed_root_guard(
            execution,
            runtime,
            admitted,
            &frontier,
            layout.entity_kind,
            &layout.locator,
            expected,
        )
        .map_err(|cause| path_union::frontier_denial(cause, field))?;
        work.charge_predicate(
            result.entity_records_examined(),
            result.matching_entity_ids_reserved(),
            result.field_reads(),
            field,
        )?;
        let mut matching = BTreeSet::new();
        for root in result.into_matching_entity_ids() {
            insert(execution, allocation, &mut matching, root, field)?;
        }
        frontier = matching;
    }
    Ok(frontier)
}
