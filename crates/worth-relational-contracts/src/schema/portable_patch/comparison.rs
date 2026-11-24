//! Compare a restored relation while its original scope owns every temporary.
use super::endpoints::append;
use super::quote::readmission_bytes;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::identity::data::EntityId;
use crate::schema::data::LoweredAspectContractPlan;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::PortableRecordAspectPatch;

pub struct ScopedEndpointComparison {
    portable: PortableRecordAspectPatch,
    _memory: Option<ExecutionMemoryReservation>,
}
impl ScopedEndpointComparison {
    /// No admitted patch, state or authority escapes this comparison phase.
    pub fn compare<E>(
        self,
        compare: impl FnOnce(PortableRecordAspectPatch) -> Result<(), E>,
    ) -> Result<(), E> {
        compare(self.portable)
    }
}

pub fn append_portable_in_scope(
    original: &PortableRecordAspectPatch,
    plan: &LoweredAspectContractPlan,
    source: EntityId,
    target: EntityId,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<ScopedEndpointComparison, Cause> {
    context.poll().map_err(Cause::from)?;
    let bytes = u64::try_from(
        original
            .clone_allocation_bytes()
            .ok_or(Cause::ChargedBytesOverflow)?,
    )
    .map_err(|_| Cause::ChargedBytesOverflow)?;
    let memory = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
        .map_err(crate::execution::denial::lease_denial)?;
    let mut operations = original.operations().to_vec();
    let mut funding = Some(memory);
    append(
        &mut operations,
        &mut funding,
        Some(plan),
        source,
        target,
        |memory, bytes| resize(context, lease, memory, bytes),
    )?;
    let portable = PortableRecordAspectPatch::new(operations);
    let bytes = readmission_bytes(
        funding
            .as_ref()
            .map_or(0, ExecutionMemoryReservation::bytes),
        &portable,
        Some(plan),
    )?;
    resize(context, lease, &mut funding, bytes)?;
    Ok(ScopedEndpointComparison {
        portable,
        _memory: funding,
    })
}
fn resize(
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    memory: &mut Option<ExecutionMemoryReservation>,
    bytes: u64,
) -> Result<(), Cause> {
    context.poll().map_err(Cause::from)?;
    match memory {
        Some(memory) => memory.resize(bytes).map_err(|cause| {
            crate::execution::denial::lease_denial(worth_execution::LeaseDenial::MemoryExhausted(
                cause,
            ))
        }),
        None => {
            *memory = Some(
                ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
                    .map_err(crate::execution::denial::lease_denial)?,
            );
            Ok(())
        }
    }
}
