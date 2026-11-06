//! Admit the actual basis and witness allocations before reconstructing meaning.
use super::*;
use std::alloc::Layout;

#[derive(Debug)]
pub(crate) enum CompositeBasisBackingDenial {
    Request(worth_execution::WorkCeilingDenial),
    CapacityOverflow,
}

pub(crate) fn admit_validated_for_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    identities: &std::sync::Mutex<RuntimeWorldIdentityIssuer>,
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
) -> Result<AdmittedCompositeRuntimeWorldBasis, CompositeBasisBackingDenial> {
    use CompositeBasisBackingDenial as Denial;
    request.consult().map_err(Denial::Request)?;
    let inner = Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(Layout::new::<AdmittedCompositeRuntimeWorldBasisInner>())
        .map_err(|_| Denial::CapacityOverflow)?
        .0
        .pad_to_align();
    let witness = Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(Layout::new::<
            AuthorityWitness<CompositeBasisAdmissionAuthorityMarker>,
        >())
        .map_err(|_| Denial::CapacityOverflow)?
        .0
        .pad_to_align();
    let bytes = inner
        .size()
        .checked_add(witness.size())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::CapacityOverflow)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(Denial::Request)?;
    let identity = identities
        .lock()
        .unwrap_or_else(|cause| cause.into_inner())
        .composite_basis(
            relational.admission_identity().clone(),
            signal.admission_identity().clone(),
            correspondence.admission_identity().clone(),
        );
    let basis = CompositeRuntimeWorldBasis::admit(relational, signal, correspondence);
    Ok(AdmittedCompositeRuntimeWorldBasis::with_backing(
        basis, identity, memory,
    ))
}
