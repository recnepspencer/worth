use worth_proof::{Artifact, AuthorityWitness, TransitionOutcome};

use super::super::basis::CanonicalBasisConstructionAuthority;
use super::super::{
    CanonicalBasisReadyArtifact, CanonicalBundleReadyArtifact,
    CanonicalDigestDerivationReadinessProofs, CanonicalDigestDerivationReady,
    CanonicalDigestInputShapeBound, CanonicalExportReadyArtifact, CanonicalRuleVersionBound,
};
use super::algorithm::{
    CanonicalDigestAlgorithmMetadata, CanonicalDomainBundleDigestAlgorithmSlot,
    CanonicalExportBundleDigestAlgorithmSlot, CanonicalSingleSequenceDigestAlgorithmSlot,
};
use super::derived::CanonicalDigestDerivationDenial;
use super::evidence::{
    CanonicalDigestBasisBundle, CanonicalDigestBasisSequence, CanonicalDigestDerivationInput,
    CanonicalDigestInputEvidence,
};
use super::material::canonical_digest_material;
use super::resource_admission::{CanonicalDigestPreparationStop, CanonicalResourceAdmission};
use super::{CanonicalDigestWorkBudget, CanonicalDigestWorkEvidence};

pub type CanonicalDigestDerivationReadyArtifact = Artifact<
    CanonicalDigestDerivationReady,
    CanonicalDigestDerivationInput,
    CanonicalDigestDerivationReadinessProofs,
    worth_proof::FreshnessScopedBasis<
        worth_proof::CurrentValidity,
        worth_proof::AssumptionBasis<CanonicalDigestAlgorithmMetadata>,
    >,
>;

pub fn admit_canonical_sequence_digest_derivation(
    sequence: CanonicalBasisReadyArtifact,
    slot: CanonicalSingleSequenceDigestAlgorithmSlot,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    admit_canonical_sequence_digest_derivation_with_budget(
        sequence,
        slot,
        CanonicalDigestWorkBudget::standard(),
    )
}

pub fn admit_canonical_sequence_digest_derivation_with_budget(
    sequence: CanonicalBasisReadyArtifact,
    slot: CanonicalSingleSequenceDigestAlgorithmSlot,
    budget: CanonicalDigestWorkBudget,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    let (sequence, _proofs, _basis) = sequence.into_parts().into_parts();
    let evidence = CanonicalDigestInputEvidence::SingleSequence(
        CanonicalDigestBasisSequence::from_owned_sequence(sequence),
    );
    admit_canonical_digest_derivation(slot.into_metadata(), evidence, budget)
}

pub fn admit_canonical_bundle_digest_derivation(
    bundle: CanonicalBundleReadyArtifact,
    slot: CanonicalDomainBundleDigestAlgorithmSlot,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    admit_canonical_bundle_digest_derivation_with_budget(
        bundle,
        slot,
        CanonicalDigestWorkBudget::standard(),
    )
}

pub fn admit_canonical_bundle_digest_derivation_with_budget(
    bundle: CanonicalBundleReadyArtifact,
    slot: CanonicalDomainBundleDigestAlgorithmSlot,
    budget: CanonicalDigestWorkBudget,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    let (bundle, _proofs, _basis) = bundle.into_parts().into_parts();
    let sequences = bundle
        .sequences()
        .iter()
        .map(|sequence| {
            let payload = sequence.payload();
            CanonicalDigestBasisSequence::new(
                payload.version().clone(),
                payload.domain(),
                payload.entries(),
                payload.cost(),
            )
        })
        .collect();
    let evidence = CanonicalDigestInputEvidence::DomainBundle(CanonicalDigestBasisBundle::new(
        bundle.version().clone(),
        sequences,
    ));

    admit_canonical_digest_derivation(slot.into_metadata(), evidence, budget)
}

pub fn admit_canonical_export_digest_derivation(
    export: CanonicalExportReadyArtifact,
    slot: CanonicalExportBundleDigestAlgorithmSlot,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    admit_canonical_export_digest_derivation_with_budget(
        export,
        slot,
        CanonicalDigestWorkBudget::standard(),
    )
}

pub fn admit_canonical_export_digest_derivation_with_budget(
    export: CanonicalExportReadyArtifact,
    slot: CanonicalExportBundleDigestAlgorithmSlot,
    budget: CanonicalDigestWorkBudget,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    let (export, _proofs, _basis) = export.into_parts().into_parts();
    let sequences = export
        .bundle()
        .sequences()
        .iter()
        .map(|sequence| {
            CanonicalDigestBasisSequence::new(
                sequence.version().clone(),
                sequence.domain(),
                sequence.entries(),
                sequence.cost(),
            )
        })
        .collect();
    let evidence = CanonicalDigestInputEvidence::ExportBundle(CanonicalDigestBasisBundle::new(
        export.bundle().version().clone(),
        sequences,
    ));

    admit_canonical_digest_derivation(slot.into_metadata(), evidence, budget)
}

fn admit_canonical_digest_derivation(
    algorithm: CanonicalDigestAlgorithmMetadata,
    evidence: CanonicalDigestInputEvidence,
    budget: CanonicalDigestWorkBudget,
) -> TransitionOutcome<CanonicalDigestDerivationReadyArtifact, CanonicalDigestDerivationDenial> {
    match prepare_canonical_digest_derivation(algorithm, evidence, budget, None) {
        Ok(ready) => TransitionOutcome::success(ready),
        Err(CanonicalDigestPreparationStop::Derivation(denial)) => {
            TransitionOutcome::denied(denial)
        }
        Err(stop) => unreachable!("ordinary codec has no resource-refusal port: {stop:?}"),
    }
}

pub(super) fn prepare_canonical_digest_derivation(
    algorithm: CanonicalDigestAlgorithmMetadata,
    evidence: CanonicalDigestInputEvidence,
    budget: CanonicalDigestWorkBudget,
    mut admission: Option<&mut CanonicalResourceAdmission<'_>>,
) -> Result<CanonicalDigestDerivationReadyArtifact, CanonicalDigestPreparationStop> {
    if let Some(admission) = admission.as_deref_mut() {
        let work = algorithm
            .id()
            .as_str()
            .len()
            .checked_add(algorithm.rule_version().as_str().len())
            .and_then(|work| work.checked_add(evidence.version().as_str().len()))
            .and_then(|work| {
                work.checked_add(match algorithm.input_domain() {
                    super::algorithm::CanonicalDigestInputDomain::Single(
                        super::super::CanonicalBasisDomain::Future(name),
                    ) => name.len(),
                    _ => 0,
                })
            })
            .and_then(|work| work.checked_add(5))
            .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
        admission(work, 0).map_err(|()| CanonicalDigestPreparationStop::ResourceRefused)?;
    }
    if !algorithm.id().is_supported() {
        return Err(CanonicalDigestPreparationStop::Derivation(
            CanonicalDigestDerivationDenial::UnsupportedAlgorithm,
        ));
    }
    if algorithm.rule_version() != evidence.version() {
        return Err(CanonicalDigestPreparationStop::Derivation(
            CanonicalDigestDerivationDenial::RuleVersionMismatch,
        ));
    }
    if algorithm.input_shape() != evidence.input_shape() {
        return Err(CanonicalDigestPreparationStop::Derivation(
            CanonicalDigestDerivationDenial::InputShapeMismatch,
        ));
    }
    if algorithm.input_domain() != evidence.input_domain() {
        return Err(CanonicalDigestPreparationStop::Derivation(
            CanonicalDigestDerivationDenial::InputDomainMismatch,
        ));
    }
    let entry_count = evidence.entry_count();
    if entry_count > budget.maximum_entry_count() {
        return Err(CanonicalDigestPreparationStop::Derivation(
            CanonicalDigestDerivationDenial::EntryLimitExceeded {
                maximum: budget.maximum_entry_count(),
                actual: entry_count,
            },
        ));
    }
    let material = canonical_digest_material(
        &algorithm,
        &evidence,
        budget.maximum_encoded_bytes(),
        admission.as_deref_mut(),
    )?;
    let work = CanonicalDigestWorkEvidence::new(
        entry_count,
        material.encoded_bytes(),
        material.allocation_bytes(),
    );

    let authority =
        AuthorityWitness::from_authority_marker(CanonicalBasisConstructionAuthority::new());
    let proofs = CanonicalDigestDerivationReadinessProofs::new(
        worth_proof::Proof::<
            CanonicalDigestInputShapeBound,
            CanonicalBasisConstructionAuthority,
        >::from_authority_witness(&authority),
        worth_proof::Proof::<CanonicalRuleVersionBound, CanonicalBasisConstructionAuthority>::from_authority_witness(
            &authority,
        ),
    );
    let input = CanonicalDigestDerivationInput::new(
        admitted_algorithm_clone(&algorithm, admission)?,
        evidence,
        material.into_bytes(),
        work,
    );

    Ok(Artifact::with_proofs_and_current_basis(
        input, proofs, algorithm, authority,
    ))
}

pub(super) fn admitted_algorithm_clone(
    algorithm: &CanonicalDigestAlgorithmMetadata,
    admission: Option<&mut CanonicalResourceAdmission<'_>>,
) -> Result<CanonicalDigestAlgorithmMetadata, CanonicalDigestPreparationStop> {
    if let Some(admission) = admission {
        let bytes = algorithm
            .id()
            .as_str()
            .len()
            .checked_add(algorithm.rule_version().as_str().len())
            .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
        let work = bytes
            .checked_add(2)
            .ok_or(CanonicalDigestPreparationStop::AccountingOverflow)?;
        admission(work, bytes).map_err(|()| CanonicalDigestPreparationStop::ResourceRefused)?;
    }
    Ok(algorithm.clone())
}
