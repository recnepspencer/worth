#[allow(unused_imports)] use crate::certification::ordinary_context::boundary_evidence::provenance::*;
use {worth_foundational::admit_requested_foundational_profile, worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path, worth_foundational::boundary_evidence_api::lower_lane, worth_foundational::derive_foundational_profile_identity, worth_foundational::foundational_boundary_evidence_provenance_layer_definitions, worth_foundational::foundational_boundary_evidence_source_basis_kind_definitions, worth_foundational::foundational_diagnostic_code, worth_foundational::foundational_diagnostic_scope, worth_foundational::foundational_profile_progression_authority, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalBoundaryEvidenceAuthorityPath, worth_foundational::FoundationalBoundaryEvidenceCanonicalDigestBasis, worth_foundational::FoundationalBoundaryEvidenceComparisonBasis, worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture, worth_foundational::FoundationalBoundaryEvidenceLocality, worth_foundational::FoundationalBoundaryEvidenceProfileBasis, worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial, worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind, worth_foundational::FoundationalBoundaryEvidenceSourceBasis, worth_foundational::FoundationalBoundaryEvidenceSourceBasisKind, worth_foundational::FoundationalBoundaryEvidenceStrategyBasis, worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment, worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::FoundationalTransitionLocator, worth_foundational::FoundationalTransitionStrategyFamily, worth_foundational::FoundationalTransitionStrategyId, worth_foundational::FoundationalTransitionStrategyIdentity, worth_foundational::FoundationalTransitionStrategyOwnershipClass, worth_foundational::FoundationalTransitionStrategySemanticName, worth_foundational::FoundationalTransitionStrategyVersion, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
use {worth_proof::TransitionOutcome};

#[test]
fn provenance_layer_definitions_are_blind_consumer_interpretable() {
    let layers = foundational_boundary_evidence_provenance_layer_definitions();
    let source_basis_kinds = foundational_boundary_evidence_source_basis_kind_definitions();

    assert_eq!(
        layers
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec![
            "source_basis",
            "authority_path",
            "strategy_basis",
            "profile_basis",
            "comparison_basis",
            "canonical_digest_basis",
            "support_context_attachment",
        ]
    );
    assert!(layers
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
    assert!(layers
        .iter()
        .all(|definition| !definition.must_not_mean().trim().is_empty()));
    assert_eq!(
        source_basis_kinds
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec!["boundary_artifact", "transition"]
    );
}

#[test]
fn provenance_common_path_builds_typed_current_context_with_explicit_layers() {
    let scope = foundational_diagnostic_scope("milestone7.provenance").expect("scope");
    let code = foundational_diagnostic_code("lineage.provenance").expect("code");

    let provenance = match worth_foundational::boundary_evidence_api::common_path::provenance()
        .current(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .comparison_basis(worth_foundational::FoundationalBoundaryEvidenceComparisonBasis::comparison(
            worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .canonical_digest_basis(digest_basis())
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope),
        )
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code),
        )
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
    {
        worth_proof::TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected provenance artifact, got {outcome:?}"),
    };

    assert_eq!(
        provenance.locality(),
        worth_foundational::FoundationalBoundaryEvidenceLocality::Current
    );
    assert_eq!(
        provenance.freshness_posture(),
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );
    assert_eq!(
        provenance.source_basis().kind(),
        worth_foundational::FoundationalBoundaryEvidenceSourceBasisKind::BoundaryArtifact
    );
    assert!(provenance.has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::SourceBasis));
    assert!(provenance.has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::AuthorityPath));
    assert!(provenance.has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::StrategyBasis));
    assert!(provenance.has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::ProfileBasis));
    assert!(provenance.has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::ComparisonBasis));
    assert!(
        provenance.has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::CanonicalDigestBasis)
    );
    assert!(provenance
        .has_layer(worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind::SupportContextAttachment));
}

#[test]
fn provenance_freshness_law_is_locality_explicit_and_fail_closed() {
    assert_eq!(
        boundary_evidence()
            .provenance()
            .replay_derived(source_basis())
            .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial::ReplayDerivedLocalityRequiresReplayFreshness
        )
    );
    assert_eq!(
        boundary_evidence()
            .provenance()
            .restored_readmitted(source_basis())
            .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial::RestoredReadmittedLocalityRequiresRestoredFreshness
        )
    );
    assert_eq!(
        boundary_evidence()
            .provenance()
            .branch_local(source_basis())
            .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial::CurrentOrBranchLocalLocalityMustNotUseReplayFreshness
        )
    );
}

#[test]
fn provenance_source_basis_family_is_explicit_across_boundary_and_transition_roots() {
    let boundary_root = match boundary_evidence()
        .provenance()
        .historical(source_basis())
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        worth_proof::TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected boundary-root provenance, got {outcome:?}"),
    };
    let transition_root = match boundary_evidence()
        .provenance()
        .historical(transition_source_basis())
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        worth_proof::TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected transition-root provenance, got {outcome:?}"),
    };

    assert_eq!(
        boundary_root.source_basis().kind(),
        worth_foundational::FoundationalBoundaryEvidenceSourceBasisKind::BoundaryArtifact
    );
    assert_eq!(
        transition_root.source_basis().kind(),
        worth_foundational::FoundationalBoundaryEvidenceSourceBasisKind::Transition
    );
    assert!(boundary_root
        .source_basis()
        .boundary_artifact_locator()
        .is_some());
    assert!(boundary_root.source_basis().transition_locator().is_none());
    assert!(transition_root
        .source_basis()
        .boundary_artifact_locator()
        .is_none());
    assert!(transition_root
        .source_basis()
        .transition_locator()
        .is_some());
}

#[test]
fn provenance_support_context_attachments_canonicalize_and_deduplicate() {
    let scope = foundational_diagnostic_scope("provenance.scope").expect("scope");
    let code = foundational_diagnostic_code("provenance.code").expect("code");
    let provenance = match boundary_evidence()
        .provenance()
        .historical(source_basis())
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope.clone()),
        )
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code.clone()),
        )
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope),
        )
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        worth_proof::TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected provenance artifact, got {outcome:?}"),
    };

    assert_eq!(
        provenance.support_context_attachments(),
        &[
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code),
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(
                foundational_diagnostic_scope("provenance.scope").expect("scope")
            ),
        ]
    );
}

#[test]
fn provenance_semantics_are_stable_across_independent_builder_orderings() {
    let scope = foundational_diagnostic_scope("provenance.independent.scope").expect("scope");
    let code = foundational_diagnostic_code("provenance.independent.code").expect("code");

    let authority_first = match boundary_evidence()
        .provenance()
        .historical(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .comparison_basis(worth_foundational::FoundationalBoundaryEvidenceComparisonBasis::comparison(
            worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .canonical_digest_basis(digest_basis())
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope.clone()),
        )
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code.clone()),
        )
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        worth_proof::TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected authority-first provenance, got {outcome:?}"),
    };

    let digest_first = match worth_foundational::boundary_evidence_api::common_path::provenance()
        .historical(source_basis())
        .canonical_digest_basis(digest_basis())
        .comparison_basis(worth_foundational::FoundationalBoundaryEvidenceComparisonBasis::comparison(
            worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .profile_basis(profile_basis())
        .strategy_basis(strategy_basis())
        .authority_path(authority_path())
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code),
        )
        .attach_support_context(
            worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope),
        )
        .with_freshness(worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        worth_proof::TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected digest-first provenance, got {outcome:?}"),
    };

    assert_eq!(authority_first, digest_first);
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase2_surface() {
    assert_eq!(
        boundary_evidence().provenance_layer_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::provenance::foundational_boundary_evidence_provenance_layer_definitions()
    );
    assert_eq!(
        boundary_evidence().source_basis_kind_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::provenance::foundational_boundary_evidence_source_basis_kind_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceProvenanceFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::provenance();
}
