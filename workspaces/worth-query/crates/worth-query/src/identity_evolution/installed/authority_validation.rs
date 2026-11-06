use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_evolution::{
    IdentityEvolutionExecutionArtifact, IdentityEvolutionExecutionFamily,
    IdentityEvolutionOutcomeFamily,
};
use crate::runtime::{WorthQueryContinuityMutationEvidence, WorthQueryContinuityOutcomeClass};

pub(super) fn semantic_evidence_identity(
    artifact: &IdentityEvolutionExecutionArtifact,
    continuity: Option<&WorthQueryContinuityMutationEvidence>,
    lifecycle_target: Option<&crate::memory_workspace::WorthQueryEntityIdentity>,
    establishing_entity_targets: &[crate::memory_workspace::WorthQueryEntityIdentity],
) -> Result<crate::evidence_identity::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let identity = semantic_identity_basis(artifact, continuity);
    let identity = match continuity {
        Some(evidence) => append_continuity_semantics(identity, evidence),
        None => identity,
    };
    let lifecycle_identity =
        lifecycle_target.map(crate::memory_workspace::WorthQueryEntityIdentity::evidence_identity);
    let establishing_target_identities = establishing_entity_targets
        .iter()
        .map(crate::memory_workspace::WorthQueryEntityIdentity::evidence_identity)
        .collect::<Vec<_>>();
    Ok(identity
        .field_evidence_identity_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("establishing_entity_target"),
            &establishing_target_identities,
        )
        .optional_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("lifecycle_target"),
            lifecycle_identity.as_ref()?,
        )
        .seal()?)
}

fn semantic_identity_basis(
    artifact: &IdentityEvolutionExecutionArtifact,
    continuity: Option<&WorthQueryContinuityMutationEvidence>,
) -> crate::evidence_identity::WorthQueryEvidenceIdentityEncoder {
    crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::InstalledDomainExecution,
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
        "installed_identity_evolution_semantics_v3",
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("evolution_family"),
        artifact.family().as_str(),
    )
    .field_shape(
        crate::evidence_identity::WorthQueryEvidenceTag::new("evolution_outcome"),
        continuity
            .map(|evidence| evidence.outcome_class().as_str())
            .unwrap_or_else(|| artifact.result_bundle().outcome_family().as_str()),
    )
    .optional_evidence_identity(
        crate::evidence_identity::WorthQueryEvidenceTag::new("prior_identity"),
        continuity.map(|evidence| evidence.prior_authoritative_identity().evidence_identity()),
    )
}

fn append_continuity_semantics(
    identity: crate::evidence_identity::WorthQueryEvidenceIdentityEncoder,
    continuity: &WorthQueryContinuityMutationEvidence,
) -> crate::evidence_identity::WorthQueryEvidenceIdentityEncoder {
    identity
        .field_evidence_identity_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("successor_identity"),
            continuity
                .successor_authoritative_identities()
                .iter()
                .map(crate::runtime::WorthQueryMutationAuthorityIdentity::evidence_identity),
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("lineage_digest"),
            continuity.lineage_digest().evidence_identity(),
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("resolution_digest"),
            continuity
                .continuity_resolution_digest()
                .evidence_identity(),
        )
}

pub(super) fn engine_matches_authority(
    artifact: &IdentityEvolutionExecutionArtifact,
    continuity: Option<&WorthQueryContinuityMutationEvidence>,
    lifecycle_target: Option<&crate::memory_workspace::WorthQueryEntityIdentity>,
) -> bool {
    if let Some(target) = lifecycle_target {
        return engine_matches_lifecycle_target(artifact, target);
    }
    let Some(continuity) = continuity else {
        return engine_matches_comparison_outcome(artifact);
    };
    continuity_shape_matches(artifact, continuity)
        && continuity_successors_match(artifact, continuity)
}

fn engine_matches_lifecycle_target(
    artifact: &IdentityEvolutionExecutionArtifact,
    target: &crate::memory_workspace::WorthQueryEntityIdentity,
) -> Result<bool, IdentityDerivationDenial> {
    let evidence = target.evidence_identity()?;
    Ok(match artifact.family() {
        IdentityEvolutionExecutionFamily::GeneratedIdentity => artifact
            .result_bundle()
            .as_generated_identity()
            .is_some_and(|result| result.authoritative_identity() == evidence.as_str()),
        IdentityEvolutionExecutionFamily::RetiredIdentity => artifact
            .result_bundle()
            .as_retired_identity()
            .is_some_and(|result| result.authoritative_identity() == evidence.as_str()),
        _ => false,
    })
}

fn engine_matches_comparison_outcome(artifact: &IdentityEvolutionExecutionArtifact) -> bool {
    artifact.family() == &IdentityEvolutionExecutionFamily::InstalledOperationComparison
        && matches!(
            artifact.result_bundle().outcome_family(),
            IdentityEvolutionOutcomeFamily::AdvisoryIdentityCandidateSet
                | IdentityEvolutionOutcomeFamily::Ambiguity
                | IdentityEvolutionOutcomeFamily::IdentityBreak
        )
}

fn continuity_shape_matches(
    artifact: &IdentityEvolutionExecutionArtifact,
    continuity: &WorthQueryContinuityMutationEvidence,
) -> bool {
    matches!(
        (artifact.family(), continuity.outcome_class()),
        (
            IdentityEvolutionExecutionFamily::DirectSuccessor,
            WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
        ) | (
            IdentityEvolutionExecutionFamily::DirectSplitSuccessors,
            WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
        ) | (
            IdentityEvolutionExecutionFamily::DirectMergeSuccessor,
            WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
        )
    ) && matches!(
        (
            artifact.result_bundle().outcome_family(),
            continuity.outcome_class()
        ),
        (
            IdentityEvolutionOutcomeFamily::SingularIdentityContinuity,
            WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
                | WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
        ) | (
            IdentityEvolutionOutcomeFamily::PluralIdentitySuccessorSet,
            WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
        )
    )
}

fn continuity_successors_match(
    artifact: &IdentityEvolutionExecutionArtifact,
    continuity: &WorthQueryContinuityMutationEvidence,
) -> bool {
    let exact = continuity
        .successor_authoritative_identities()
        .iter()
        .map(|identity| identity.evidence_identity().as_str())
        .collect::<Vec<_>>();
    match continuity.outcome_class() {
        WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
        | WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
            artifact
                .result_bundle()
                .as_singular_identity_continuity()
                .is_some_and(|result| exact.as_slice() == [result.authoritative_identity()])
        }
        WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors => artifact
            .result_bundle()
            .as_plural_identity_successor_set()
            .is_some_and(|result| {
                result
                    .successor_identities()
                    .iter()
                    .map(String::as_str)
                    .eq(exact)
            }),
        _ => false,
    }
}
