//! Canonical grammar and digest belong to Foundation; Query carries request cancellation.
use super::sealing::WorthQuerySignedCreationPreparationDenial as Denial;
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalDigestAdmissionStop, CanonicalDigestAlgorithmId, CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

pub(in crate::domain_computation::primary_graph::application_attempt) fn derive(
    entries: Vec<CanonicalBasisEntry>,
    domain: CanonicalBasisDomain,
    rule: &str,
    request: &ExecutionRequest<'_, '_>,
    _context: &mut MapKernelContext<'_, '_>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), Denial> {
    let check = || {
        request
            .check_live()
            .map_err(|cause| execution(WorkCeilingDenial::Stopped(cause)))
    };
    check()?;
    let version = CanonicalizationRuleVersion::new(rule).ok_or(Denial::Canonicalization)?;
    let basis = prepare_canonical_basis_sequence(version, domain, entries)
        .into_result()
        .map_err(Denial::CanonicalBasis)?;
    let ready = canonicalization()
        .digest()
        .for_sequence(basis, CanonicalDigestAlgorithmId::sha256())
        .into_result()
        .map_err(Denial::CanonicalDigest)?;
    check()?;
    let derived = canonicalization()
        .digest()
        .derive_with_admission(ready, &mut |_, _| check())
        .map_err(digest_stop)?;
    check()?;
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
