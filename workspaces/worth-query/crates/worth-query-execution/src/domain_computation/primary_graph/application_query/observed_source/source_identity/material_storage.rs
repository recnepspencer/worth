//! Actual shared interner backing keeps every original managed-source grant.
use super::*;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) struct ManagedInternerStorage<'a, 'request, 'authority> {
    pub(super) execution: &'a ExecutionRequest<'request, 'authority>,
    pub(super) allocation: &'a Arc<SourceFactsAllocation>,
}
impl ManagedInternerStorage<'_, '_, '_> {
    pub(super) fn work(&self, units: usize) -> Result<(), Cause> {
        let units = u64::try_from(units).map_err(|_| Cause::WorkCounterOverflow)?;
        let mut result = None;
        let outcome = self.execution.run_native_with_checkpoint(|context| {
            result = Some(if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            });
        });
        match (outcome, result) {
            (Ok(_), Some(result)) => result.map_err(Into::into),
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(cause.into()),
            (Err(cause), _) => Err(cause.into()),
            (Ok(_), None) => unreachable!("completed interner checkpoint"),
        }
    }

    pub(super) fn lookup(&self, count: usize) -> Result<(), Cause> {
        // A lookup compares no key more than once; this bound requires no
        // private BTree fanout or height contract.
        self.work(
            count
                .checked_add(1)
                .and_then(|count| {
                    count.checked_mul(size_of::<WorthQueryObservedSourceCoordinate>())
                })
                .ok_or(Cause::WorkCounterOverflow)?,
        )
    }

    pub(super) fn comparison(
        &self,
        left: &WorthQueryObservedSourceFootprint,
        right: &WorthQueryObservedSourceFootprint,
    ) -> Result<(), Cause> {
        admit_footprint_comparison(left, right, &mut |units| self.work(units))
    }

    pub(super) fn scratch(
        &self,
        count: usize,
    ) -> Result<Vec<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        self.work(count.checked_mul(2).ok_or(Cause::WorkCounterOverflow)?)?;
        self.allocation
            .claim_slice::<Arc<WorthQueryObservedSourceMeaning>>(self.execution, count)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        Ok(values)
    }

    pub(super) fn meaning(&self) -> Result<(), Cause> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<WorthQueryObservedSourceMeaning>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        self.allocation.claim(self.execution, bytes)
    }

    pub(super) fn prepare_container(
        &self,
        state: &mut WorthQueryObservedSourceMeaningRegistryState,
        coordinate: &WorthQueryObservedSourceCoordinate,
    ) -> Result<Option<Vec<Weak<WorthQueryObservedSourceMeaning>>>, Cause> {
        self.lookup(state.meanings.len())?;
        let existing = state.meanings.get(coordinate);
        if existing.is_some_and(|entries| entries.len() < entries.capacity()) {
            return Ok(None);
        }
        let replacement = existing
            .map_or(Some(1), |entries| entries.len().checked_add(1))
            .ok_or(Cause::ChargedBytesOverflow)?;
        // The map and coordinate vectors share backing across meanings. Their
        // original grants belong to the registry state, not any single Arc.
        self.work(
            state
                .material_allocations
                .len()
                .checked_mul(2)
                .ok_or(Cause::WorkCounterOverflow)?,
        )?;
        if !state
            .material_allocations
            .iter()
            .any(|grant| Arc::ptr_eq(grant, self.allocation))
        {
            let count = state
                .material_allocations
                .len()
                .checked_add(1)
                .ok_or(Cause::ChargedBytesOverflow)?;
            self.allocation
                .claim_slice::<Arc<SourceFactsAllocation>>(self.execution, count)?;
            let mut owners = Vec::new();
            owners
                .try_reserve_exact(count)
                .map_err(|_| Cause::AllocationFailed)?;
            owners.extend(state.material_allocations.iter().cloned());
            owners.push(Arc::clone(self.allocation));
            state.material_allocations = owners;
        }
        self.allocation
            .claim_slice::<Weak<WorthQueryObservedSourceMeaning>>(self.execution, replacement)?;
        self.lookup(state.meanings.len())?;
        if let Some(entries) = state.meanings.get_mut(coordinate) {
            entries
                .try_reserve_exact(1)
                .map_err(|_| Cause::AllocationFailed)?;
            Ok(None)
        } else {
            self.allocation.claim_tree::<(
                WorthQueryObservedSourceCoordinate,
                Vec<Weak<WorthQueryObservedSourceMeaning>>,
            )>(self.execution, state.meanings.len())?;
            let mut entries = Vec::new();
            entries
                .try_reserve_exact(1)
                .map_err(|_| Cause::AllocationFailed)?;
            Ok(Some(entries))
        }
    }
}

impl WorthQueryObservedSourceMeaningRegistryState {
    pub(super) fn take_empty_material_storage(
        &mut self,
    ) -> Option<(
        BTreeMap<WorthQueryObservedSourceCoordinate, Vec<Weak<WorthQueryObservedSourceMeaning>>>,
        Vec<Arc<SourceFactsAllocation>>,
    )> {
        self.meanings.is_empty().then(|| {
            (
                std::mem::take(&mut self.meanings),
                std::mem::take(&mut self.material_allocations),
            )
        })
    }
}

/// The same source-meaning comparison admission for a managed selection's
/// borrowed dependencies; this neither allocates nor touches their old grant.
pub(in crate::domain_computation::primary_graph::application_query) fn admit_footprint_comparison(
    left: &WorthQueryObservedSourceFootprint,
    right: &WorthQueryObservedSourceFootprint,
    admit: &mut impl FnMut(usize) -> Result<(), Cause>,
) -> Result<(), Cause> {
    for footprint in [left, right] {
        if let Some(managed) = footprint
            .root_selection
            .as_ref()
            .and_then(|selection| selection.managed_index.as_ref())
        {
            admit(managed.dependencies.len())?;
        }
    }
    let mut visits = 0usize;
    for facts in [left, right]
        .into_iter()
        .flat_map(|source| source.fact_sources())
    {
        visits = visits
            .checked_add(facts.entities.len())
            .and_then(|n| n.checked_add(facts.aspects.len()))
            .and_then(|n| n.checked_add(facts.adjacencies.len()))
            .ok_or(Cause::WorkCounterOverflow)?;
    }
    admit(visits)?;
    let mut work = 0usize;
    for facts in [left, right]
        .into_iter()
        .flat_map(|source| source.fact_sources())
    {
        work = work
            .checked_add(
                facts
                    .entities
                    .len()
                    .checked_mul(size_of::<worth_relational::facade::identity::EntityId>())
                    .ok_or(Cause::WorkCounterOverflow)?,
            )
            .ok_or(Cause::WorkCounterOverflow)?;
        for aspect in facts.aspects {
            work = work
                .checked_add(size_of_val(aspect))
                .and_then(|n| n.checked_add(aspect.entity_name.len()))
                .and_then(|n| n.checked_add(aspect.aspect.as_str().len()))
                .and_then(|n| n.checked_add(aspect.field.as_str().len()))
                .ok_or(Cause::WorkCounterOverflow)?;
        }
        for adjacency in facts.adjacencies {
            work = work
                .checked_add(size_of_val(adjacency))
                .and_then(|n| n.checked_add(adjacency.owned_capacity_bytes()))
                .ok_or(Cause::WorkCounterOverflow)?;
        }
    }
    admit(work)
}
