//! Genuine birth streams choose construction only after checking their full keys.
use super::*;
use crate::history::data::CanonicalRecordAllocation;
use worth_execution::ExecutionRequestArrayBuilder;

fn ordered_counts<'a, E>(
    records: impl IntoIterator<Item = &'a RecordRef>,
    mut visit: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<(usize, usize)>, E> {
    let (mut entity, mut relation) = (None, None);
    let (mut entities, mut relations) = (0, 0);
    for record in records {
        visit(size_of::<RecordRef>())?;
        match record {
            RecordRef::Entity(id) => {
                if entity.is_some_and(|previous| previous >= id) {
                    return Ok(None);
                }
                entity = Some(id);
                entities += 1;
            }
            RecordRef::Relation(id) => {
                if relation.is_some_and(|previous| previous >= id) {
                    return Ok(None);
                }
                relation = Some(id);
                relations += 1;
            }
        }
    }
    Ok(Some((entities, relations)))
}

impl AuthoredSourceInputs {
    pub(super) fn allocations(
        execution: ExecutionRequest<'_, '_>,
        budget: &CustomPreparationBudget,
        allocations: &[CanonicalRecordAllocation],
        before: &InvariantStateView<'_>,
        after: &InvariantStateView<'_>,
    ) -> Result<Self, Denial> {
        let Some((entities, relations)) = ordered_counts(
            allocations.iter().map(CanonicalRecordAllocation::record),
            |units| admission::work(budget, units),
        )?
        else {
            let mut result = Self {
                records: ExecutionOrderedMap::default(),
                incidence: ExecutionOrderedMap::default(),
            };
            for allocation in allocations {
                admission::work(budget, 1)?;
                match allocation.record() {
                    record @ RecordRef::Entity(_) => {
                        result.record(execution, budget, record, Use::LIFECYCLE)?
                    }
                    RecordRef::Relation(id) => {
                        result.relation(execution, budget, *id, before, after)?
                    }
                }
            }
            return Ok(result);
        };
        let mut incidence = ExecutionRequestArrayBuilder::allocate(execution, relations)
            .map_err(admission::array)?;
        for allocation in allocations {
            admission::work(budget, 1)?;
            if let RecordRef::Relation(id) = allocation.record() {
                incidence
                    .push((*id, super::incidence::observe(*id, before, after, budget)?))
                    .map_err(admission::array)?;
            }
        }
        let incidence = incidence.seal().map_err(admission::array)?;
        let units = entities
            .checked_mul(2 * size_of::<RecordRef>() + size_of::<Use>() + 16 * size_of::<usize>())
            .and_then(|n| {
                relations
                    .checked_mul(
                        2 * size_of::<RelationId>()
                            + size_of::<AuthoredSourceIncidence>()
                            + 16 * size_of::<usize>(),
                    )
                    .and_then(|m| n.checked_add(m))
            })
            .ok_or_else(|| {
                admission::storage(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            })?;
        admission::work(budget, units)?;
        let records = ExecutionOrderedMap::from_sorted_native_for_request(
            entities,
            allocations
                .iter()
                .filter_map(|allocation| match allocation.record() {
                    record @ RecordRef::Entity(_) => Some((record.clone(), Use::LIFECYCLE)),
                    RecordRef::Relation(_) => None,
                }),
            execution,
        )
        .map_err(admission::map)?
        .ok_or(Denial::ForeignSource)?;
        let incidence = ExecutionOrderedMap::from_sorted_native_for_request(
            relations,
            incidence.elements().iter().copied(),
            execution,
        )
        .map_err(admission::map)?
        .ok_or(Denial::ForeignSource)?;
        Ok(Self { records, incidence })
    }
}

#[cfg(test)]
mod tests;
