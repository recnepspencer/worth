pub(crate) use {worth_foundational::admit_requested_foundational_profile, worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path, worth_foundational::boundary_evidence_api::lower_lane, worth_foundational::derive_foundational_profile_identity, worth_foundational::foundational_boundary_evidence_provenance_layer_definitions, worth_foundational::foundational_boundary_evidence_source_basis_kind_definitions, worth_foundational::foundational_diagnostic_code, worth_foundational::foundational_diagnostic_scope, worth_foundational::foundational_profile_progression_authority, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId, worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalBoundaryEvidenceAuthorityPath, worth_foundational::FoundationalBoundaryEvidenceCanonicalDigestBasis, worth_foundational::FoundationalBoundaryEvidenceComparisonBasis, worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture, worth_foundational::FoundationalBoundaryEvidenceLocality, worth_foundational::FoundationalBoundaryEvidenceProfileBasis, worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial, worth_foundational::FoundationalBoundaryEvidenceProvenanceLayerKind, worth_foundational::FoundationalBoundaryEvidenceSourceBasis, worth_foundational::FoundationalBoundaryEvidenceSourceBasisKind, worth_foundational::FoundationalBoundaryEvidenceStrategyBasis, worth_foundational::FoundationalBoundaryEvidenceSupportContextAttachment, worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis, worth_foundational::FoundationalCommitParentageLocator, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::FoundationalTransitionLocator, worth_foundational::FoundationalTransitionStrategyFamily, worth_foundational::FoundationalTransitionStrategyId, worth_foundational::FoundationalTransitionStrategyIdentity, worth_foundational::FoundationalTransitionStrategyOwnershipClass, worth_foundational::FoundationalTransitionStrategySemanticName, worth_foundational::FoundationalTransitionStrategyVersion, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn source_basis() -> FoundationalBoundaryEvidenceSourceBasis {
    worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(worth_foundational::BoundaryArtifactLocator::new(
        worth_foundational::BoundaryArtifactId::new(41),
        worth_foundational::BoundaryArtifactField::Basis,
    ))
}

pub(crate) fn transition_source_basis() -> FoundationalBoundaryEvidenceSourceBasis {
    worth_foundational::FoundationalBoundaryEvidenceSourceBasis::transition(
        worth_foundational::FoundationalTransitionLocator::CommitParentage(worth_foundational::FoundationalCommitParentageLocator::new(
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(17)),
            worth_foundational::FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(29)),
        )),
    )
}

pub(crate) fn authority_path() -> FoundationalBoundaryEvidenceAuthorityPath {
    worth_foundational::FoundationalBoundaryEvidenceAuthorityPath::transition(
        worth_foundational::FoundationalTransitionLocator::CommitParentage(worth_foundational::FoundationalCommitParentageLocator::new(
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(7)),
            worth_foundational::FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(19)),
        )),
    )
}

pub(crate) fn strategy_basis() -> FoundationalBoundaryEvidenceStrategyBasis {
    worth_foundational::FoundationalBoundaryEvidenceStrategyBasis::strategy(
        worth_foundational::FoundationalTransitionStrategyIdentity::new(
            worth_foundational::FoundationalTransitionStrategyId::new(worth_foundational::BoundaryHandle::new(11)),
            worth_foundational::FoundationalTransitionStrategyFamily::new("merge").expect("family"),
            worth_foundational::FoundationalTransitionStrategySemanticName::new("promotion").expect("name"),
            worth_foundational::FoundationalTransitionStrategyVersion::new("v1").expect("version"),
            worth_foundational::FoundationalTransitionStrategyOwnershipClass::RuntimeBuiltIn,
        ),
    )
}

pub(crate) fn admitted_profile() -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
        support_posture: worth_foundational::SupportPostureProfile::SupportReady,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::EvidenceBacked,
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
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    };
    let identity = match derive_foundational_profile_identity(
        worth_foundational::CanonicalizationRuleVersion::new("m7.phase2.provenance")
            .expect("version"),
        &admitted,
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        outcome => panic!("expected profile identity, got {outcome:?}"),
    };

    worth_foundational::FoundationalBoundaryEvidenceProfileBasis::profile(identity)
}

pub(crate) fn digest_basis() -> FoundationalBoundaryEvidenceCanonicalDigestBasis {
    worth_foundational::FoundationalBoundaryEvidenceCanonicalDigestBasis::digest(
        profile_basis().identity().digest().clone(),
    )
}