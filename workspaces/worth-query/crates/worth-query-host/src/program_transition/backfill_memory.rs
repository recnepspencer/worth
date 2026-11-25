//! Exact pre-allocation retained-memory sizing and reservation custody for
//! declarative transition backfill planning.

use std::alloc::Layout;
use std::mem::size_of;
use std::sync::Mutex;

use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelStop,
    WorkCeilingDenial,
};
use worth_foundational::facade::{
    AspectContract, AspectFieldLocator, AspectKey, AspectShape, AspectValue,
    AuthoritativeRecordAspectState, FieldKey, PortableAspectPatchOperation, StructAspectValue,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
        WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
    },
};
use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::backfill::AspectDefaultGroup;

/// Exact contiguous array allocation size for `count` elements of `T`.
pub(super) fn vec_backing_bytes<T>(
    count: usize,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    if count == 0 {
        return Ok(0);
    }
    let layout = Layout::array::<T>(count).map_err(|_| retained_bytes_overflow())?;
    checked_usize(layout.size())
}

/// Full B-tree node capacity upper bound for `std::collections::BTreeMap<K, V>`
/// (`B = 6`, 11 entries per node, 12 child pointers on internal nodes).
/// Even for `count == 1`, covers the full initial leaf node allocation.
pub(super) fn btree_map_retained_bytes<K, V>(
    count: usize,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    if count == 0 {
        return Ok(0);
    }
    let pair = checked_add(
        checked_usize(size_of::<K>())?,
        checked_usize(size_of::<V>())?,
    )?;
    let leaf_node = checked_add(checked_mul(pair, 11)?, 64)?;
    if count <= 11 {
        return Ok(leaf_node);
    }
    let internal_node = checked_add(
        leaf_node,
        checked_mul(checked_usize(size_of::<usize>())?, 12)?,
    )?;
    let max_nodes = checked_add(checked_usize(count / 5)?, 2)?;
    checked_mul(max_nodes, internal_node)
}

/// Pre-allocation byte cost for constructing `planned_field(aspect, field)` and
/// cloning `AspectValue` into the pre-sized `Vec<(AspectFieldLocator, AspectValue)>`.
pub(super) fn planned_locator_and_value_bytes(
    aspect: &str,
    field: &str,
    value: &AspectValue,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let aspect_heap = checked_usize(aspect.len())?;
    let field_heap = checked_usize(field.len())?;
    let locator_heap = checked_add(
        aspect_heap,
        checked_add(checked_usize(size_of::<FieldKey>())?, field_heap)?,
    )?;
    let value_heap = value
        .clone_retained_storage_bytes()
        .ok_or_else(retained_bytes_overflow)?;
    checked_add(locator_heap, checked_usize(value_heap)?)
}

/// Pre-allocation byte cost for one `AspectDefaultGroup`: its cloned
/// `AspectKey`, cloned `AspectContract`, and exact `locators` vector backing.
pub(super) fn contract_group_retained_bytes(
    aspect_key: &AspectKey,
    contract: &AspectContract,
    locator_count: usize,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let key_heap = checked_usize(aspect_key.as_str().len())?;
    let contract_heap = checked_usize(contract.owned_allocation_capacity_bytes())?;
    let vec_bytes = vec_backing_bytes::<(AspectFieldLocator, AspectValue)>(locator_count)?;
    checked_add(key_heap, checked_add(contract_heap, vec_bytes)?)
}

/// Pre-allocation byte cost for a formatted batch name string, its
/// `WorkerIntentBatch` container, and exact `MutationIntent` backing.
pub(super) fn batch_allocation_bytes(
    name_len: usize,
    exact_intents: usize,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        checked_usize(name_len)?,
        checked_add(
            checked_usize(size_of::<WorkerIntentBatch>())?,
            vec_backing_bytes::<MutationIntent>(exact_intents)?,
        )?,
    )
}

/// Exact pre-allocation retained byte cost for all intents cloned and
/// constructed for one entity row, including `StructAspectValue`'s exact
/// foundational tree/key/value `Arc` backing (`sorted_construction_storage_bytes`),
/// exact `attach_ops` vector backing, and full `BTreeMap` node capacity for any
/// `UpdateEntityFieldsIntent`.
pub(super) fn entity_backfill_retained_bytes(
    aspect_state: Option<&AuthoritativeRecordAspectState>,
    groups: &[AspectDefaultGroup],
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let mut total = 0u64;
    let mut attach_count = 0usize;
    let mut update_field_count = 0usize;

    for group in groups {
        let already_attached =
            aspect_state.is_some_and(|state| state.get(&group.aspect_key).is_some());
        if already_attached {
            for (locator, value) in &group.locators {
                update_field_count = update_field_count
                    .checked_add(1)
                    .ok_or_else(retained_bytes_overflow)?;
                let entry_bytes = checked_add(
                    locator_retained_bytes(locator)?,
                    aspect_value_retained_bytes(value)?,
                )?;
                total = checked_add(total, entry_bytes)?;
            }
        } else {
            attach_count = attach_count
                .checked_add(1)
                .ok_or_else(retained_bytes_overflow)?;
            let op_bytes = checked_add(
                checked_usize(group.contract.owned_allocation_capacity_bytes())?,
                checked_usize(size_of::<PortableAspectPatchOperation>())?,
            )?;
            total = checked_add(total, op_bytes)?;
            match group.contract.shape() {
                AspectShape::Struct(_) => {
                    let struct_bytes =
                        struct_attachment_retained_bytes(group.locators.iter().map(
                            |(locator, value)| (locator.field_path().fields().first(), value),
                        ))?;
                    total = checked_add(total, struct_bytes)?;
                }
                AspectShape::Scalar(_) | AspectShape::Opaque(_) | AspectShape::Reference(_) => {
                    for (_, value) in &group.locators {
                        total = checked_add(total, aspect_value_retained_bytes(value)?)?;
                    }
                }
            }
        }
    }

    if attach_count > 0 {
        let attach_vec = vec_backing_bytes::<PortableAspectPatchOperation>(attach_count)?;
        let intent_bytes = checked_add(
            attach_vec,
            checked_add(
                checked_usize(size_of::<ApplyEntityAspectPatchIntent>())?,
                checked_usize(size_of::<MutationIntent>())?,
            )?,
        )?;
        total = checked_add(total, intent_bytes)?;
    }
    if update_field_count > 0 {
        let map_nodes =
            btree_map_retained_bytes::<AspectFieldLocator, AspectValue>(update_field_count)?;
        let intent_bytes = checked_add(
            map_nodes,
            checked_add(
                checked_usize(size_of::<UpdateEntityFieldsIntent>())?,
                checked_usize(size_of::<MutationIntent>())?,
            )?,
        )?;
        total = checked_add(total, intent_bytes)?;
    }
    Ok(total)
}

pub(super) fn reserve_retained(
    request: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    retained_memory: &Mutex<Vec<ExecutionMemoryReservation>>,
    bytes: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if bytes == 0 {
        return Ok(());
    }
    if let Some(reservation) = request
        .reserve_retained_memory_in_stage(context, bytes)
        .map_err(work_ceiling_denial)?
        && let Ok(mut guard) = retained_memory.lock()
    {
        guard.push(reservation);
    }
    Ok(())
}

pub(super) fn charge_work(
    context: &mut MapKernelContext<'_, '_>,
    units: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    context
        .checkpoint(units)
        .map_err(|stop| work_ceiling_denial(WorkCeilingDenial::Stopped(stop)))
}

pub(super) fn work_ceiling_denial(
    cause: WorkCeilingDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match cause {
        WorkCeilingDenial::Stopped(MapKernelStop::Cancelled) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                kind: WorthQueryProviderSessionControlStopKind::Cancelled,
            }
        }
        WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                kind: WorthQueryProviderSessionControlStopKind::TimedOut,
            }
        }
        WorkCeilingDenial::Stopped(_) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                kind: WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None,
                    policy_ancestor: None,
                },
            }
        }
        WorkCeilingDenial::Admission(_) => {
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                kind: WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                    partition_identity: None,
                    policy_ancestor: None,
                },
            }
        }
        WorkCeilingDenial::Panicked => {
            WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
        }
    };
    WorthQueryPrimaryGraphInstallationDenial::new(
        kind,
        format!("transition backfill stopped by execution budget: {cause:?}"),
    )
}

pub(super) fn retained_bytes_overflow() -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
            kind: WorthQueryProviderSessionDenialKind::ExecutionResource {
                denial: WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                partition_identity: None,
                policy_ancestor: None,
            },
        },
        "transition backfill retained-memory calculation overflowed",
    )
}

fn aspect_value_retained_bytes(
    value: &AspectValue,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let heap = value
        .clone_retained_storage_bytes()
        .ok_or_else(retained_bytes_overflow)?;
    checked_add(
        checked_usize(heap)?,
        checked_usize(size_of::<AspectValue>())?,
    )
}

fn locator_retained_bytes(
    locator: &AspectFieldLocator,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        checked_usize(locator.owned_allocation_capacity_bytes())?,
        checked_usize(size_of::<AspectFieldLocator>())?,
    )
}

fn checked_usize(value: usize) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    u64::try_from(value).map_err(|_| retained_bytes_overflow())
}

fn checked_add(a: u64, b: u64) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    a.checked_add(b).ok_or_else(retained_bytes_overflow)
}

fn struct_attachment_retained_bytes<'a>(
    fields: impl ExactSizeIterator<Item = (Option<&'a FieldKey>, &'a AspectValue)>,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let field_count = fields.len();
    let tree_backing = StructAspectValue::sorted_construction_storage_bytes(field_count)
        .ok_or_else(retained_bytes_overflow)?;
    let staging_slice = vec_backing_bytes::<(FieldKey, AspectValue)>(field_count)?;
    let mut total = checked_add(checked_usize(tree_backing)?, staging_slice)?;
    for (field_key, value) in fields {
        let field_heap = field_key.map_or(0, |field| field.owned_allocation_capacity_bytes());
        let entry_bytes = checked_add(
            checked_usize(field_heap)?,
            aspect_value_retained_bytes(value)?,
        )?;
        total = checked_add(total, entry_bytes)?;
    }
    Ok(total)
}

fn checked_mul(a: u64, b: u64) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    a.checked_mul(b).ok_or_else(retained_bytes_overflow)
}

pub(super) fn count_distinct_runs<T, K: Eq>(sorted: &[T], mut key: impl FnMut(&T) -> K) -> usize {
    let mut count = 0usize;
    let mut prev: Option<K> = None;
    for item in sorted {
        let current = key(item);
        if prev.as_ref() != Some(&current) {
            count += 1;
            prev = Some(current);
        }
    }
    count
}

pub(super) fn count_batch_intents<'a>(
    states: impl IntoIterator<Item = Option<&'a AuthoritativeRecordAspectState>>,
    groups: &[AspectDefaultGroup],
) -> Result<usize, WorthQueryPrimaryGraphInstallationDenial> {
    let mut total = 0usize;
    for state in states {
        let mut has_attach = false;
        let mut has_update = false;
        for group in groups {
            if state.is_some_and(|s| s.get(&group.aspect_key).is_some()) {
                has_update = true;
            } else {
                has_attach = true;
            }
        }
        total = total
            .checked_add(usize::from(has_attach) + usize::from(has_update))
            .ok_or_else(retained_bytes_overflow)?;
    }
    Ok(total)
}

pub(super) fn schema_denial(detail: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember,
        detail,
    )
}

#[cfg(test)]
#[path = "backfill_memory_tests.rs"]
mod tests;
