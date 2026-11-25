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
    AspectContract, AspectFieldLocator, AspectShape, AspectValue, AuthoritativeRecordAspectState,
    FieldKey, PortableAspectPatchOperation, StructAspectValue,
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

/// Per-entry node overhead for `std::collections::BTreeMap` (B=6 internal
/// pointers, parent pointer, length/height metadata, and slack up to 2x).
fn btree_entry_bytes<K, V>() -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let pair = checked_add(
        checked_usize(size_of::<K>())?,
        checked_usize(size_of::<V>())?,
    )?;
    checked_add(checked_mul(pair, 2)?, 64)
}

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

/// Byte cost of adding one default reference to the per-entity grouping map.
pub(super) fn by_entity_entry_bytes() -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        btree_entry_bytes::<&'static str, Vec<usize>>()?,
        checked_usize(size_of::<usize>())?,
    )
}

/// Pre-allocation byte cost for constructing `planned_field(aspect, field)` and
/// storing `(AspectKey, AspectFieldLocator, AspectValue)` in the aspect map.
pub(super) fn planned_group_entry_bytes(
    aspect: &str,
    field: &str,
    value: &AspectValue,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    // AspectFieldLocator owns AspectKey string + FieldPath Vec<FieldKey> (1 key).
    let aspect_heap = checked_usize(aspect.len())?;
    let field_heap = checked_usize(field.len())?;
    let locator_bytes = checked_add(
        checked_usize(size_of::<AspectFieldLocator>())?,
        checked_add(
            aspect_heap,
            checked_add(checked_usize(size_of::<FieldKey>())?, field_heap)?,
        )?,
    )?;
    // Map owns an additional cloned AspectKey string plus cloned AspectValue.
    let key_bytes = checked_add(
        checked_usize(size_of::<worth_foundational::facade::AspectKey>())?,
        aspect_heap,
    )?;
    let value_bytes = aspect_value_retained_bytes(value)?;
    let map_overhead = btree_entry_bytes::<AspectFieldLocator, AspectValue>()?;
    checked_add(
        locator_bytes,
        checked_add(key_bytes, checked_add(value_bytes, map_overhead)?)?,
    )
}

pub(super) fn contract_group_retained_bytes(
    contract: &AspectContract,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        checked_usize(contract.owned_allocation_capacity_bytes())?,
        checked_usize(size_of::<AspectDefaultGroup>())?,
    )
}

/// Pre-allocation byte cost for a formatted batch name string, its
/// `WorkerIntentBatch` container, and exact `MutationIntent` backing.
pub(super) fn batch_allocation_bytes(
    name_len: usize,
    estimated_intents: usize,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        checked_usize(name_len)?,
        checked_add(
            checked_usize(size_of::<WorkerIntentBatch>())?,
            vec_backing_bytes::<MutationIntent>(estimated_intents)?,
        )?,
    )
}

/// Exact pre-allocation retained byte cost for all intents cloned and
/// constructed for one entity row, including `StructAspectValue`'s exact
/// foundational tree/key/value `Arc` backing (`sorted_construction_storage_bytes`)
/// and staging slice.
pub(super) fn entity_backfill_retained_bytes(
    aspect_state: Option<&AuthoritativeRecordAspectState>,
    groups: &[AspectDefaultGroup],
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let mut total = 0u64;
    let mut attach_count = 0usize;
    let mut has_update = false;

    for group in groups {
        let already_attached =
            aspect_state.is_some_and(|state| state.get(&group.aspect_key).is_some());
        if already_attached {
            has_update = true;
            for (locator, value) in &group.locators {
                let entry_bytes = checked_add(
                    locator_retained_bytes(locator)?,
                    checked_add(
                        aspect_value_retained_bytes(value)?,
                        btree_entry_bytes::<AspectFieldLocator, AspectValue>()?,
                    )?,
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
                    for value in group.locators.values() {
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
    if has_update {
        let intent_bytes = checked_add(
            checked_usize(size_of::<UpdateEntityFieldsIntent>())?,
            checked_usize(size_of::<MutationIntent>())?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{AspectValue, FieldKey, InternedString};

    fn make_fields(field_count: usize, payload_len: usize) -> Vec<(FieldKey, AspectValue)> {
        (0..field_count)
            .map(|idx| {
                (
                    FieldKey::new(format!("Field{idx:03}")).unwrap(),
                    AspectValue::String(InternedString::Raw("a".repeat(payload_len))),
                )
            })
            .collect()
    }

    #[test]
    fn entity_backfill_bytes_covers_struct_tree_backing_and_string_payloads() {
        let small = make_fields(1, 8);
        let wide = make_fields(32, 8);
        let large = make_fields(1, 64 * 1024);

        let b_small =
            struct_attachment_retained_bytes(small.iter().map(|(k, v)| (Some(k), v))).unwrap();
        let b_wide =
            struct_attachment_retained_bytes(wide.iter().map(|(k, v)| (Some(k), v))).unwrap();
        let b_large =
            struct_attachment_retained_bytes(large.iter().map(|(k, v)| (Some(k), v))).unwrap();

        let exact_tree_32 =
            StructAspectValue::sorted_construction_storage_bytes(32).unwrap() as u64;
        assert!(
            b_wide >= exact_tree_32,
            "32-field struct charge ({b_wide}) must cover exact StructAspectValue tree backing ({exact_tree_32})"
        );
        assert!(b_wide > b_small * 10);
        assert!(
            b_large >= 64 * 1024,
            "64 KiB string payload must be charged in full ({b_large})"
        );

        let group_large = planned_group_entry_bytes(
            "TaskPriority",
            "Priority",
            &AspectValue::String(InternedString::Raw("x".repeat(32 * 1024))),
        )
        .unwrap();
        assert!(group_large >= 32 * 1024);

        let batch_bytes = batch_allocation_bytes(128, 16).unwrap();
        assert!(batch_bytes >= 128 + (16 * size_of::<MutationIntent>()) as u64);
    }
}
