//! Keep source recovery failures typed without minting any recovered receipt.
use super::Denial;
use worth_relational::facade::durability::RelationalAssociatedEffectRefusal as Native;
pub(super) fn denial(cause: Native) -> Denial {
    match cause {
        Native::Recovery(cause) => Denial::NativeRecovery(Box::new(cause)),
        Native::OwnerUnavailable
        | Native::AssociatedMemory { .. }
        | Native::MissingDiscovery { .. }
        | Native::ForeignRuntime
        | Native::CatalogSubjectChanged
        | Native::MissingSettlementEvidence
        | Native::SubjectMismatch => Denial::NativeSourceUnavailable,
    }
}
pub(super) fn read_subject(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    original: &worth_runtime_world::facade::WorldOriginalStateObservation<'_>,
    subject: &worth_relational::facade::durability::RelationalAssociatedEffectSubject,
) -> Result<worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect, Denial> {
    let association = subject.association().ok_or(Denial::OriginalMismatch)?;
    if !original
        .matches_native_association(request, association)
        .map_err(|_| Denial::NativeSourceUnavailable)?
    {
        return Err(Denial::OriginalMismatch);
    }
    native
        .durability_authority()
        .readmit_associated_effect(request, subject)
        .map_err(denial)
}
