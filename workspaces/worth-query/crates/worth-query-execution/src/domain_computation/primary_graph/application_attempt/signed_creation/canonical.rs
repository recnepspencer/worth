//! Foundation owns canonical meaning; the active Query request admits its work and storage.
use super::sealing::WorthQuerySignedCreationPreparationDenial as Denial;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, MapKernelContext, WorkCeilingDenial,
};
use worth_foundational::facade::{
    canonicalization, prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisPreparationStop, CanonicalDigestAdmissionStop,
    CanonicalDigestAlgorithmId, CanonicalDigestWorkBudget, CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

pub(in crate::domain_computation::primary_graph::application_attempt) fn derive(
    entries: Vec<CanonicalBasisEntry>,
    domain: CanonicalBasisDomain,
    rule: &str,
    maximum_bytes: usize,
    request: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), Denial> {
    let version = CanonicalizationRuleVersion::new(rule).ok_or(Denial::Canonicalization)?;
    let budget = CanonicalDigestWorkBudget::new(
        u32::try_from(entries.len()).map_err(|_| Denial::Canonicalization)?,
        maximum_bytes,
    )
    .ok_or(Denial::Canonicalization)?;
    let mut memory = request
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(execution)?
        .map_err(|cause| execution(WorkCeilingDenial::Admission(cause)))?;
    let mut admit = |work: usize, bytes: usize| {
        let work = u64::try_from(work).map_err(|_| Denial::RepresentationCapacity)?;
        context
            .checkpoint(work)
            .map_err(|cause| execution(WorkCeilingDenial::Stopped(cause)))?;
        let total = memory
            .bytes()
            .checked_add(u64::try_from(bytes).map_err(|_| Denial::RepresentationCapacity)?)
            .ok_or(Denial::RepresentationCapacity)?;
        memory.resize(total).map_err(|cause| {
            execution(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                cause,
            )))
        })
    };
    let basis =
        prepare_owned_canonical_basis_sequence_admitted(version, domain, entries, &mut admit)
            .map_err(|cause| match cause {
                CanonicalBasisPreparationStop::Resource { cause, .. } => cause,
                CanonicalBasisPreparationStop::Construction(cause) => Denial::CanonicalBasis(cause),
                CanonicalBasisPreparationStop::AccountingOverflow => {
                    Denial::CanonicalAccountingOverflow
                }
                CanonicalBasisPreparationStop::UnsupportedSortImplementation => {
                    Denial::CanonicalSortUnsupported
                }
            })?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_admission(
            basis,
            CanonicalDigestAlgorithmId::sha256(),
            budget,
            &mut admit,
        )
        .map_err(digest_stop)?;
    let derived = canonicalization()
        .digest()
        .derive_with_admission(ready, &mut admit)
        .map_err(digest_stop)?;
    Ok((
        *derived.value().bytes(),
        WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    ))
}
fn digest_stop(cause: CanonicalDigestAdmissionStop<Denial>) -> Denial {
    match cause {
        CanonicalDigestAdmissionStop::Resource(cause) => cause,
        CanonicalDigestAdmissionStop::Derivation(cause) => Denial::CanonicalDigest(cause),
        CanonicalDigestAdmissionStop::AccountingOverflow => Denial::CanonicalAccountingOverflow,
        CanonicalDigestAdmissionStop::AllocationUnavailable => {
            Denial::CanonicalAllocationUnavailable
        }
    }
}
fn execution(cause: WorkCeilingDenial) -> Denial {
    Denial::Execution(
        crate::domain_computation::primary_graph::application_contribution::advancement_denial(
            cause,
        ),
    )
}
