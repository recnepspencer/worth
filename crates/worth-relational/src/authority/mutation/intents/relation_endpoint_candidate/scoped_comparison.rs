//! Compare a restored relation while its original scope owns every temporary.
use super::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::PortableRecordAspectPatch;

pub(crate) struct ScopedEndpointComparison {
    portable: PortableRecordAspectPatch,
    _memory: Option<ExecutionMemoryReservation>,
}
impl ScopedEndpointComparison {
    /// No admitted patch, state or authority escapes this comparison phase.
    pub(crate) fn compare<E>(
        self,
        compare: impl FnOnce(PortableRecordAspectPatch) -> Result<(), E>,
    ) -> Result<(), E> {
        compare(self.portable)
    }
}

pub(crate) fn append_portable_in_scope(
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
    let candidate = FieldCandidate {
        operations: original.operations().to_vec(),
        funding: Some(memory),
    };
    let mut candidate = append(candidate, Some(plan), source, target, |memory, bytes| {
        resize(context, lease, memory, bytes)
    })?;
    let portable = PortableRecordAspectPatch::new(std::mem::take(&mut candidate.operations));
    let bytes = super::super::field_authoring_candidate::readmission_bytes(
        candidate
            .funding
            .as_ref()
            .map_or(0, ExecutionMemoryReservation::bytes),
        &portable,
        Some(plan),
    )?;
    resize(context, lease, &mut candidate.funding, bytes)?;
    Ok(ScopedEndpointComparison {
        portable,
        _memory: candidate.funding,
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
