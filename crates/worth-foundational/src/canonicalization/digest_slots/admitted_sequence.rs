use super::{
    admission::prepare_canonical_digest_derivation,
    evidence::{CanonicalDigestBasisSequence, CanonicalDigestInputEvidence},
    resource_admission::{CanonicalDigestAdmissionStop, CanonicalDigestPreparationStop},
};
use crate::canonicalization::{
    CanonicalBasisReadyArtifact, CanonicalDigestAlgorithmId, CanonicalDigestWorkBudget,
};

pub(in crate::canonicalization) fn prepare_sequence_with_admission<Stop>(
    sequence: CanonicalBasisReadyArtifact,
    algorithm_id: CanonicalDigestAlgorithmId,
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<super::CanonicalDigestDerivationReadyArtifact, CanonicalDigestAdmissionStop<Stop>> {
    let mut refusal = None;
    let mut admission = |work, bytes| {
        admit(work, bytes).map_err(|stop| {
            refusal = Some(stop);
        })
    };
    let result = prepare_sequence(sequence, algorithm_id, budget, &mut admission);
    result.map_err(|stop| stop.preserve(refusal))
}

fn prepare_sequence(
    sequence: CanonicalBasisReadyArtifact,
    algorithm_id: CanonicalDigestAlgorithmId,
    budget: CanonicalDigestWorkBudget,
    admit: &mut impl FnMut(usize, usize) -> Result<(), ()>,
) -> Result<super::CanonicalDigestDerivationReadyArtifact, CanonicalDigestPreparationStop> {
    let bytes = sequence.payload().version().as_str().len();
    let work = bytes
        .checked_add(1)
        .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
    admit(work, bytes).map_err(|()| CanonicalDigestPreparationStop::ResourceRefused)?;
    let slot = crate::canonicalization::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        algorithm_id,
        sequence.payload().domain(),
        sequence.payload().version().clone(),
    );
    let (sequence, _, _) = sequence.into_parts().into_parts();
    let evidence = CanonicalDigestInputEvidence::SingleSequence(
        CanonicalDigestBasisSequence::from_owned_sequence(sequence),
    );
    prepare_canonical_digest_derivation(slot.into_metadata(), evidence, budget, Some(admit))
}
