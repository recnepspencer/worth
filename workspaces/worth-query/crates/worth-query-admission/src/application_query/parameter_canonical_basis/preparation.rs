//! Resource admission around the existing parameter basis and digest engines.
use super::*;
use worth_foundational::facade::{
    prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisPreparationStop,
    CanonicalDigestAdmissionStop,
};

#[derive(Debug)]
pub(in crate::application_query) enum ParameterBasisPreparationStop<Stop> {
    Admission(Stop),
    Preparation {
        cause: Stop,
        quote: worth_foundational::facade::CanonicalBasisPreparationQuote,
    },
    Derivation(CanonicalDigestDerivationDenial),
    AccountingOverflow,
}

pub(in crate::application_query) fn prepare_parameter_basis_admitted<Stop>(
    bindings: &[(&'static str, AspectValue)],
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<WorthQueryApplicationParameterCanonicalArtifact, ParameterBasisPreparationStop<Stop>> {
    use ParameterBasisPreparationStop::{AccountingOverflow, Admission};
    let entries = prepared_entries(bindings, admit)?;
    admit(PARAMETER_RULE.len(), PARAMETER_RULE.len()).map_err(Admission)?;
    let version = CanonicalizationRuleVersion::new(PARAMETER_RULE)
        .expect("the fixed application-query parameter rule is valid");
    let basis = prepare_owned_canonical_basis_sequence_admitted(
        version,
        CanonicalBasisDomain::Future(PARAMETER_DOMAIN),
        entries,
        &mut *admit,
    )
    .map_err(|stop| match stop {
        CanonicalBasisPreparationStop::Resource { cause, quote } => {
            ParameterBasisPreparationStop::Preparation { cause, quote }
        }
        CanonicalBasisPreparationStop::Construction(_) => unreachable!("parameter loci are unique"),
        CanonicalBasisPreparationStop::AccountingOverflow
        | CanonicalBasisPreparationStop::UnsupportedSortImplementation => AccountingOverflow,
    })?;
    admit(basis.payload().entries().len(), 0).map_err(Admission)?;
    // The digest consumes a copy while the admitted parameter retains its basis.
    let bytes = basis.payload().owned_allocation_capacity_bytes();
    admit(bytes, bytes).map_err(Admission)?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_admission(
            basis.clone(),
            CanonicalDigestAlgorithmId::sha256(),
            budget,
            admit,
        )
        .map_err(digest_stop)?;
    let derived = canonicalization()
        .digest()
        .derive_with_admission(ready, admit)
        .map_err(digest_stop)?;
    Ok(WorthQueryApplicationParameterCanonicalArtifact {
        basis,
        digest: CanonicalDigestId::new(*derived.value().bytes()),
        work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    })
}

fn digest_stop<Stop>(
    stop: CanonicalDigestAdmissionStop<Stop>,
) -> ParameterBasisPreparationStop<Stop> {
    match stop {
        CanonicalDigestAdmissionStop::Resource(stop) => {
            ParameterBasisPreparationStop::Admission(stop)
        }
        CanonicalDigestAdmissionStop::Derivation(stop) => {
            ParameterBasisPreparationStop::Derivation(stop)
        }
        CanonicalDigestAdmissionStop::AccountingOverflow
        | CanonicalDigestAdmissionStop::AllocationUnavailable => {
            ParameterBasisPreparationStop::AccountingOverflow
        }
    }
}
