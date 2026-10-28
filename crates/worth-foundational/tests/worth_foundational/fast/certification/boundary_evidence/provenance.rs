use worth_foundational::{
    admit_requested_foundational_profile, boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane},
    derive_foundational_profile_identity,
    foundational_boundary_evidence_provenance_layer_definitions,
    foundational_boundary_evidence_source_basis_kind_definitions, foundational_diagnostic_code,
    foundational_diagnostic_scope, foundational_profile_progression_authority,
    request_foundational_profile_set, AdmissionReadinessProfile, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle, CanonicalEquivalenceBasis,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalBoundaryEvidenceAuthorityPath, FoundationalBoundaryEvidenceCanonicalDigestBasis,
    FoundationalBoundaryEvidenceComparisonBasis, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLocality, FoundationalBoundaryEvidenceProfileBasis,
    FoundationalBoundaryEvidenceProvenanceConstructionDenial,
    FoundationalBoundaryEvidenceProvenanceLayerKind, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryEvidenceSourceBasisKind, FoundationalBoundaryEvidenceStrategyBasis,
    FoundationalBoundaryEvidenceSupportContextAttachment, FoundationalCommitId,
    FoundationalCommitParentBasis, FoundationalCommitParentageLocator, FoundationalProfileSet,
    FoundationalProfileSetInput, FoundationalTransitionLocator,
    FoundationalTransitionStrategyFamily, FoundationalTransitionStrategyId,
    FoundationalTransitionStrategyIdentity, FoundationalTransitionStrategyOwnershipClass,
    FoundationalTransitionStrategySemanticName, FoundationalTransitionStrategyVersion,
    RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

pub(crate) fn source_basis() -> FoundationalBoundaryEvidenceSourceBasis {
    FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(41),
        BoundaryArtifactField::Basis,
    ))
}

fn transition_source_basis() -> FoundationalBoundaryEvidenceSourceBasis {
    FoundationalBoundaryEvidenceSourceBasis::transition(
        FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
            FoundationalCommitId::new(BoundaryHandle::new(17)),
            FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(29)),
        )),
    )
}

pub(crate) fn authority_path() -> FoundationalBoundaryEvidenceAuthorityPath {
    FoundationalBoundaryEvidenceAuthorityPath::transition(
        FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
            FoundationalCommitId::new(BoundaryHandle::new(7)),
            FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(19)),
        )),
    )
}

pub(crate) fn strategy_basis() -> FoundationalBoundaryEvidenceStrategyBasis {
    FoundationalBoundaryEvidenceStrategyBasis::strategy(
        FoundationalTransitionStrategyIdentity::new(
            FoundationalTransitionStrategyId::new(BoundaryHandle::new(11)),
            FoundationalTransitionStrategyFamily::new("merge").expect("family"),
            FoundationalTransitionStrategySemanticName::new("promotion").expect("name"),
            FoundationalTransitionStrategyVersion::new("v1").expect("version"),
            FoundationalTransitionStrategyOwnershipClass::RuntimeBuiltIn,
        ),
    )
}

fn admitted_profile() -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("profile")
}

pub(crate) fn profile_basis() -> FoundationalBoundaryEvidenceProfileBasis {
    let profile = admitted_profile();
    let requested = request_foundational_profile_set(profile);
    let admitted = match admit_requested_foundational_profile(
        requested,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    };
    let identity = match derive_foundational_profile_identity(
        worth_foundational::CanonicalizationRuleVersion::new("m7.phase2.provenance")
            .expect("version"),
        &admitted,
    ) {
        TransitionOutcome::Success(identity) => identity,
        outcome => panic!("expected profile identity, got {outcome:?}"),
    };

    FoundationalBoundaryEvidenceProfileBasis::profile(identity)
}

pub(crate) fn digest_basis() -> FoundationalBoundaryEvidenceCanonicalDigestBasis {
    FoundationalBoundaryEvidenceCanonicalDigestBasis::digest(
        profile_basis().identity().digest().clone(),
    )
}

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

    let provenance = match common_path::provenance()
        .current(source_basis())
        .authority_path(authority_path())
        .strategy_basis(strategy_basis())
        .profile_basis(profile_basis())
        .comparison_basis(FoundationalBoundaryEvidenceComparisonBasis::comparison(
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .canonical_digest_basis(digest_basis())
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope),
        )
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code),
        )
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
    {
        TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected provenance artifact, got {outcome:?}"),
    };

    assert_eq!(
        provenance.locality(),
        FoundationalBoundaryEvidenceLocality::Current
    );
    assert_eq!(
        provenance.freshness_posture(),
        FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained
    );
    assert_eq!(
        provenance.source_basis().kind(),
        FoundationalBoundaryEvidenceSourceBasisKind::BoundaryArtifact
    );
    assert!(provenance.has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::SourceBasis));
    assert!(provenance.has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::AuthorityPath));
    assert!(provenance.has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::StrategyBasis));
    assert!(provenance.has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::ProfileBasis));
    assert!(provenance.has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::ComparisonBasis));
    assert!(
        provenance.has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::CanonicalDigestBasis)
    );
    assert!(provenance
        .has_layer(FoundationalBoundaryEvidenceProvenanceLayerKind::SupportContextAttachment));
}

#[test]
fn provenance_freshness_law_is_locality_explicit_and_fail_closed() {
    assert_eq!(
        boundary_evidence()
            .provenance()
            .replay_derived(source_basis())
            .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceProvenanceConstructionDenial::ReplayDerivedLocalityRequiresReplayFreshness
        )
    );
    assert_eq!(
        boundary_evidence()
            .provenance()
            .restored_readmitted(source_basis())
            .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceProvenanceConstructionDenial::RestoredReadmittedLocalityRequiresRestoredFreshness
        )
    );
    assert_eq!(
        boundary_evidence()
            .provenance()
            .branch_local(source_basis())
            .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay),
        TransitionOutcome::Denied(
            FoundationalBoundaryEvidenceProvenanceConstructionDenial::CurrentOrBranchLocalLocalityMustNotUseReplayFreshness
        )
    );
}

#[test]
fn provenance_source_basis_family_is_explicit_across_boundary_and_transition_roots() {
    let boundary_root = match boundary_evidence()
        .provenance()
        .historical(source_basis())
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected boundary-root provenance, got {outcome:?}"),
    };
    let transition_root = match boundary_evidence()
        .provenance()
        .historical(transition_source_basis())
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected transition-root provenance, got {outcome:?}"),
    };

    assert_eq!(
        boundary_root.source_basis().kind(),
        FoundationalBoundaryEvidenceSourceBasisKind::BoundaryArtifact
    );
    assert_eq!(
        transition_root.source_basis().kind(),
        FoundationalBoundaryEvidenceSourceBasisKind::Transition
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
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope.clone()),
        )
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code.clone()),
        )
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope),
        )
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected provenance artifact, got {outcome:?}"),
    };

    assert_eq!(
        provenance.support_context_attachments(),
        &[
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code),
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(
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
        .comparison_basis(FoundationalBoundaryEvidenceComparisonBasis::comparison(
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .canonical_digest_basis(digest_basis())
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope.clone()),
        )
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code.clone()),
        )
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected authority-first provenance, got {outcome:?}"),
    };

    let digest_first = match common_path::provenance()
        .historical(source_basis())
        .canonical_digest_basis(digest_basis())
        .comparison_basis(FoundationalBoundaryEvidenceComparisonBasis::comparison(
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
        ))
        .profile_basis(profile_basis())
        .strategy_basis(strategy_basis())
        .authority_path(authority_path())
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_code(code),
        )
        .attach_support_context(
            FoundationalBoundaryEvidenceSupportContextAttachment::diagnostic_scope(scope),
        )
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
    {
        TransitionOutcome::Success(provenance) => provenance,
        outcome => panic!("expected digest-first provenance, got {outcome:?}"),
    };

    assert_eq!(authority_first, digest_first);
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase2_surface() {
    assert_eq!(
        boundary_evidence().provenance_layer_definitions(),
        lower_lane::provenance::foundational_boundary_evidence_provenance_layer_definitions()
    );
    assert_eq!(
        boundary_evidence().source_basis_kind_definitions(),
        lower_lane::provenance::foundational_boundary_evidence_source_basis_kind_definitions()
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceProvenanceFrontDoor =
        common_path::provenance();
}
