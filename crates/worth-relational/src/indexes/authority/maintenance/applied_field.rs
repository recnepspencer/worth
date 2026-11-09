//! Data-only field entries qualified to one immutable Native applied image.
//! A selected C generation remains the seed; no generation is issued at D.
use super::{field_delta, request_payment, root_view::IndexRootView};
use crate::{
    branch::RelationalBranchRoot,
    execution::{request_stage::checkpoint, RelationalExecutionDenialCause as Cause},
    identity::data::{EntityId, VersionId},
    indexes::data::{
        DerivedIndexEntryMap, DerivedIndexGeneration, DerivedIndexGenerationId, DerivedIndexId,
        DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
        DerivedIndexMaintenanceDenialKind as Denial,
    },
    storage::data::AuthoritativeFieldComparisonKey,
    transactions::data::RecordRef,
};
use std::sync::{Arc, Mutex};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest,
};
use worth_foundational::facade::AspectFieldLocator;

type Entries = DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, EntityId>;
type Key = (DerivedIndexId, DerivedIndexGenerationId);

#[derive(Debug)]
pub(crate) struct AppliedFieldEntries {
    pub(crate) entries: Entries,
    _seed: Arc<DerivedIndexGeneration>,
    _memory: ExecutionMemoryReservation,
}

#[derive(Debug)]
pub(crate) struct AppliedFieldImage {
    before: Arc<RelationalBranchRoot>,
    after: Arc<RelationalBranchRoot>,
    version: VersionId,
    changed: Vec<EntityId>,
    previous: Option<Arc<Self>>,
    cache: Mutex<(u64, ExecutionOrderedMap<Key, Arc<AppliedFieldEntries>>)>,
    _memory: ExecutionMemoryReservation,
}
impl AppliedFieldImage {
    pub(crate) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        before: Arc<RelationalBranchRoot>,
        after: Arc<RelationalBranchRoot>,
        version: VersionId,
        records: &[RecordRef],
        previous: Option<Arc<Self>>,
    ) -> Result<Arc<Self>, Cause> {
        let count = execution
            .run_native_with_checkpoint(|context| {
                checkpoint(
                    context,
                    (records.len() as u64).saturating_mul(2).saturating_add(1),
                )?;
                Ok::<_, worth_execution::MapKernelStop>(
                    records
                        .iter()
                        .filter(|record| matches!(record, RecordRef::Entity(_)))
                        .count(),
                )
            })
            .map_err(Cause::from)?
            .0
            .map_err(Cause::from)?;
        let bytes = count
            .checked_mul(std::mem::size_of::<EntityId>())
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = ExecutionMemoryReservation::reserve_in_scope(
            crate::authority::commit::pipeline::request_lease(execution),
            bytes as u64,
        )
        .map_err(crate::execution::denial::lease_denial)?;
        let mut changed = Vec::new();
        changed
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        for record in records {
            if let RecordRef::Entity(id) = record {
                changed.push(*id);
            }
        }
        Ok(Arc::new(Self {
            before,
            after,
            version,
            changed,
            previous,
            cache: Mutex::new((0, ExecutionOrderedMap::default())),
            _memory: memory,
        }))
    }

    pub(crate) fn entries(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: Key,
        locator: &AspectFieldLocator,
        seed: &Arc<DerivedIndexGeneration>,
    ) -> Result<Arc<AppliedFieldEntries>, DerivedIndexMaintenanceDenial> {
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: Some(0),
            maximum_derived_rows: None,
        };
        let (prepared, counts) = request_payment::run(execution, budget, |work| {
            let mut path = Vec::new();
            let mut cursor = self;
            let mut entries;
            loop {
                let cache = cursor
                    .cache
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .1
                    .clone();
                work.lookup(cache.len(), 1)?;
                let cached = cache.get(&key).cloned();
                if let Some(cached) = cached {
                    if path.is_empty() {
                        return Ok(Err(cached));
                    }
                    entries = cached.entries.clone();
                    break;
                }
                work.grow_vec(&mut path)?;
                path.push(cursor);
                match cursor.previous.as_deref() {
                    Some(previous) => cursor = previous,
                    None => {
                        let crate::indexes::data::DerivedIndexEntries::EntityField(selected) =
                            &seed.entries
                        else {
                            return Err(Denial::GenerationKindMismatch(key.0));
                        };
                        entries = selected.clone();
                        break;
                    }
                }
            }
            for image in path.into_iter().rev() {
                let before = image.previous.as_ref().map_or_else(
                    || IndexRootView::new(Arc::clone(&image.before)),
                    |prior| IndexRootView::applied(Arc::clone(&prior.after), prior.version),
                );
                let after = IndexRootView::applied(Arc::clone(&image.after), image.version);
                field_delta::entity(
                    execution,
                    &mut entries,
                    locator,
                    &before,
                    &after,
                    &image.changed,
                    work,
                )?;
            }
            work.prepare(
                1,
                (std::mem::size_of::<AppliedFieldEntries>() + 2 * std::mem::size_of::<usize>())
                    as u64,
            )?;
            Ok(Ok(entries))
        })?;
        let (result, memory) = prepared.into_funded_parts();
        match result {
            Err(cached) => Ok(cached),
            Ok(entries) => {
                // Publication of an immutable accelerator only. Never hold this
                // lock while reading Native rows or consulting request control.
                let value = Arc::new(AppliedFieldEntries {
                    entries,
                    _seed: Arc::clone(seed),
                    _memory: memory,
                });
                // Persistent snapshots let allocation and request checks run
                // before map access, without any cache lock being held.
                loop {
                    crate::execution::check_request(execution).map_err(|cause| {
                        DerivedIndexMaintenanceDenial {
                            kind: Denial::Execution(cause),
                            work: counts,
                        }
                    })?;
                    let (epoch, mut entries) = self
                        .cache
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .clone();
                    execution
                        .run_native_with_checkpoint(|context| {
                            checkpoint(
                                context,
                                u64::from(usize::BITS - entries.len().max(1).leading_zeros()) * 11
                                    + 1,
                            )
                        })
                        .map_err(Cause::from)
                        .and_then(|result| result.0.map_err(Cause::from))
                        .map_err(|cause| DerivedIndexMaintenanceDenial {
                            kind: Denial::Execution(cause),
                            work: counts,
                        })?;
                    if let Some(cached) = entries.get(&key) {
                        return Ok(Arc::clone(cached));
                    }
                    entries
                        .insert_cloned_native_for_request(&key, &value, *execution)
                        .map_err(|cause| {
                            let cause = match cause {
                                ExecutionOrderedMapDenial::Request(cause) => cause.into(),
                                ExecutionOrderedMapDenial::CapacityOverflow => {
                                    Cause::ChargedBytesOverflow
                                }
                                ExecutionOrderedMapDenial::Allocation(cause) => match cause
                                    .classify()
                                {
                                    worth_execution::ExecutionAllocationDenialClass::Request(
                                        cause,
                                    ) => cause.into(),
                                    worth_execution::ExecutionAllocationDenialClass::Physical(
                                        _,
                                    ) => Cause::AllocationFailed,
                                },
                            };
                            DerivedIndexMaintenanceDenial {
                                kind: Denial::Execution(cause),
                                work: counts,
                            }
                        })?;
                    let next = epoch.checked_add(1).ok_or(DerivedIndexMaintenanceDenial {
                        kind: Denial::Execution(Cause::WorkCounterOverflow),
                        work: counts,
                    })?;
                    let mut cache = self
                        .cache
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner());
                    if cache.0 == epoch {
                        *cache = (next, entries);
                        return Ok(value);
                    }
                }
            }
        }
    }
}
