use worth_proof::TransitionOutcome;

use super::super::{
    admit_canonical_bundle_digest_derivation, admit_canonical_bundle_digest_derivation_with_budget,
    admit_canonical_export_digest_derivation, admit_canonical_export_digest_derivation_with_budget,
    admit_canonical_sequence_digest_derivation,
    admit_canonical_sequence_digest_derivation_with_budget, derive_canonical_digest,
    CanonicalBasisReadyArtifact, CanonicalBundleReadyArtifact, CanonicalDerivedDigest,
    CanonicalDigestAlgorithmId, CanonicalDigestDerivationDenial,
    CanonicalDigestDerivationReadyArtifact, CanonicalDigestWorkBudget,
    CanonicalExportReadyArtifact,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CanonicalDigestFrontDoor;

impl CanonicalDigestFrontDoor {
    pub fn derive_with_admission<Stop>(
        self,
        ready: CanonicalDigestDerivationReadyArtifact,
        admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
    ) -> Result<CanonicalDerivedDigest, super::super::CanonicalDigestAdmissionStop<Stop>> {
        super::super::derive_canonical_digest_with_admission(ready, admit)
    }

    /// Runs the ordinary grammar with before-allocation resource admission.
    pub fn for_sequence_with_admission<Stop>(
        self,
        sequence: CanonicalBasisReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
        budget: CanonicalDigestWorkBudget,
        admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
    ) -> Result<
        CanonicalDigestDerivationReadyArtifact,
        super::super::CanonicalDigestAdmissionStop<Stop>,
    > {
        super::super::digest_slots::prepare_sequence_with_admission(
            sequence,
            algorithm_id,
            budget,
            admit,
        )
    }

    pub fn for_sequence(
        self,
        sequence: CanonicalBasisReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
    ) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial>
    {
        let slot = super::super::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            algorithm_id,
            sequence.payload().domain(),
            sequence.payload().version().clone(),
        );
        admit_canonical_sequence_digest_derivation(sequence, slot)
    }

    pub fn for_sequence_with_budget(
        self,
        sequence: CanonicalBasisReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
        budget: CanonicalDigestWorkBudget,
    ) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial>
    {
        let slot = super::super::CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
            algorithm_id,
            sequence.payload().domain(),
            sequence.payload().version().clone(),
        );
        admit_canonical_sequence_digest_derivation_with_budget(sequence, slot, budget)
    }

    pub fn for_bundle(
        self,
        bundle: CanonicalBundleReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
    ) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial>
    {
        let slot = super::super::CanonicalDomainBundleDigestAlgorithmSlot::domain_bundle(
            algorithm_id,
            bundle.payload().version().clone(),
        );
        admit_canonical_bundle_digest_derivation(bundle, slot)
    }

    pub fn for_bundle_with_budget(
        self,
        bundle: CanonicalBundleReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
        budget: CanonicalDigestWorkBudget,
    ) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial>
    {
        let slot = super::super::CanonicalDomainBundleDigestAlgorithmSlot::domain_bundle(
            algorithm_id,
            bundle.payload().version().clone(),
        );
        admit_canonical_bundle_digest_derivation_with_budget(bundle, slot, budget)
    }

    pub fn for_export(
        self,
        export: CanonicalExportReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
    ) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial>
    {
        let slot = super::super::CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
            algorithm_id,
            export.payload().bundle().version().clone(),
        );
        admit_canonical_export_digest_derivation(export, slot)
    }

    pub fn for_export_with_budget(
        self,
        export: CanonicalExportReadyArtifact,
        algorithm_id: CanonicalDigestAlgorithmId,
        budget: CanonicalDigestWorkBudget,
    ) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial>
    {
        let slot = super::super::CanonicalExportBundleDigestAlgorithmSlot::export_bundle(
            algorithm_id,
            export.payload().bundle().version().clone(),
        );
        admit_canonical_export_digest_derivation_with_budget(export, slot, budget)
    }

    pub fn derive(self, ready: CanonicalDigestDerivationReadyArtifact) -> CanonicalDerivedDigest {
        derive_canonical_digest(ready)
    }
}
