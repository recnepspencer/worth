//! Exact pre-allocation retained-memory sizing and reservation custody for
//! declarative transition backfill planning.

use std::mem::size_of;
use std::sync::Mutex;

use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelStop,
    WorkCeilingDenial,
};
use worth_foundational::facade::{
    AspectContract, AspectFieldLocator, AspectKey, AspectValue, AuthoritativeRecordAspectState,
    FieldKey, PortableAspectPatchOperation,
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

const BTREE_NODE_OVERHEAD_BYTES: u64 = 64;
const VEC_SLOT_OVERHEAD_BYTES: u64 = 32;

pub(super) fn group_entry_retained_bytes(
    locator: &AspectFieldLocator,
    value: &AspectValue,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        locator_retained_bytes(locator)?,
        checked_add(
            aspect_value_retained_bytes(value)?,
            checked_add(
                aspect_key_retained_bytes(locator.aspect().aspect_key())?,
                BTREE_NODE_OVERHEAD_BYTES,
            )?,
        )?,
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

pub(super) fn batch_header_retained_bytes(
    batch_name: &str,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        checked_usize(batch_name.len())?,
        checked_usize(size_of::<WorkerIntentBatch>())?,
    )
}

pub(super) fn entity_backfill_retained_bytes(
    aspect_state: Option<&AuthoritativeRecordAspectState>,
    groups: &[AspectDefaultGroup],
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    let mut total = 0u64;
    let mut has_attach = false;
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
                        BTREE_NODE_OVERHEAD_BYTES,
                    )?,
                )?;
                total = checked_add(total, entry_bytes)?;
            }
        } else {
            has_attach = true;
            let op_bytes = checked_add(
                checked_usize(group.contract.owned_allocation_capacity_bytes())?,
                checked_usize(size_of::<PortableAspectPatchOperation>())?,
            )?;
            total = checked_add(total, op_bytes)?;
            for (locator, value) in &group.locators {
                let field_bytes = locator
                    .field_path()
                    .fields()
                    .first()
                    .map_or(0, |field| field.as_str().len());
                let entry_bytes = checked_add(
                    checked_usize(field_bytes)?,
                    checked_add(
                        checked_usize(size_of::<FieldKey>())?,
                        checked_add(aspect_value_retained_bytes(value)?, VEC_SLOT_OVERHEAD_BYTES)?,
                    )?,
                )?;
                total = checked_add(total, entry_bytes)?;
            }
        }
    }

    if has_attach {
        let intent_bytes = checked_add(
            checked_usize(size_of::<ApplyEntityAspectPatchIntent>())?,
            checked_usize(size_of::<MutationIntent>())?,
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

fn aspect_key_retained_bytes(
    key: &AspectKey,
) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    checked_add(
        checked_usize(key.owned_allocation_capacity_bytes())?,
        checked_usize(size_of::<AspectKey>())?,
    )
}

fn checked_usize(value: usize) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    u64::try_from(value).map_err(|_| retained_bytes_overflow())
}

fn checked_add(a: u64, b: u64) -> Result<u64, WorthQueryPrimaryGraphInstallationDenial> {
    a.checked_add(b).ok_or_else(retained_bytes_overflow)
}

fn retained_bytes_overflow() -> WorthQueryPrimaryGraphInstallationDenial {
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
