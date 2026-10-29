pub use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, CanonicalizationRuleVersion,
    FoundationalPerformanceCounterName, RetentionDeliveryProfile, ScalarAspectType,
};
pub use worth_query::facade::domain;

pub struct CandidateArtifactFamily;

impl domain::WorthQueryArtifactFamily for CandidateArtifactFamily {
    const SEMANTIC_FAMILY: &'static str = "WORTH.tests.artifact-workflow.candidates";
}

pub fn candidate_contract() -> domain::WorthQueryPortableArtifactContract {
    domain::WorthQueryPortableArtifactContract::declare::<CandidateArtifactFamily>(
        domain::WorthQueryArtifactSchemaVersion::new(1),
        domain::WorthQueryArtifactProtocolVersion::new(1),
    )
    .identity(
        domain::WorthQueryArtifactContentIdentityContract::owner_canonical_projection(
            "WORTH.tests.artifact-workflow.projection",
            CanonicalizationRuleVersion::new("artifact-workflow-v1").unwrap(),
        ),
    )
    .ownership(domain::WorthQueryArtifactOwnershipContract::domain_payload(
        "WORTH.tests.geometry",
        "WORTH.tests.artifact-workflow.provider",
    ))
    .occurrence(domain::WorthQueryArtifactOccurrenceContract::independent_per_execution())
    .evidence(domain::WorthQueryArtifactEvidenceContract::new(
        "artifact-workflow-basis",
        "artifact-workflow-provenance",
        "artifact-workflow-dependency",
        "artifact-workflow-invalidation",
        "artifact-workflow-equivalence",
    ))
    .reproducibility(domain::WorthQueryArtifactReproducibilityContract::new(
        domain::WorthQueryArtifactReproducibilityClass::ExactDeterministic,
        domain::WorthQueryArtifactDeterminismPosture::Deterministic,
        domain::WorthQueryArtifactComparisonAuthority::ExactCanonicalValue,
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    ))
    .search(domain::WorthQueryCandidateSearchContract::not_applicable())
    .convergence(domain::WorthQueryConvergenceContract::NotIterative)
    .transformation(domain::WorthQueryTransformationEvidenceContract::not_a_transformation())
    .access_path(domain::WorthQueryArtifactAccessPathContract::native(
        domain::WorthQueryArtifactNativeAccessContract::new(
            candidate_layout_contract(),
            domain::WorthQueryArtifactRowBatchPosture::Borrowed,
            Some(domain::WorthQueryArtifactChunkContract::bounded(64)),
            [
                domain::WorthQueryArtifactBulkProjectionContract::new(
                    "candidate-summary-v1",
                    [candidate_id(), candidate_score()],
                    domain::WorthQueryArtifactNativeAlignment::new(8),
                    [candidate_id_contract(), candidate_score_contract()],
                ),
                domain::WorthQueryArtifactBulkProjectionContract::new(
                    "candidate-provenance-v1",
                    [candidate_token(), candidate_target(), candidate_content()],
                    domain::WorthQueryArtifactNativeAlignment::new(8),
                    [candidate_signature_contract()],
                ),
            ],
            domain::WorthQueryArtifactScalarFallbackPosture::admitted(64, 32),
        ),
    ))
    .carriage(domain::WorthQueryArtifactCarriageContract::new(
        domain::WorthQueryArtifactMovePosture::Required,
        domain::WorthQueryArtifactBorrowPosture::SharedReadOnly,
        domain::WorthQueryArtifactClonePosture::Forbidden,
        domain::WorthQueryArtifactProviderTransferPosture::MoveOwnership,
        domain::WorthQueryArtifactSerializationPosture::CanonicalProjectionOnly,
    ))
    .lifecycle(domain::WorthQueryArtifactLifecycleContract::Retained)
    .counters(
        domain::WorthQueryStructuralCounterContract::required_foundation(
            counter("artifact-bytes"),
            counter("artifact-elements"),
            counter("artifact-work"),
        ),
    )
    .decisions(domain::WorthQueryDecisionRecordContract::not_required())
    .governance(domain::WorthQueryArtifactGovernanceContract::new(
        ["workflow-internal"],
        domain::WorthQueryArtifactClassification::Internal,
        domain::WorthQueryArtifactRedactionPosture::CanonicalProjectionOnly,
        RetentionDeliveryProfile::Ephemeral,
        domain::WorthQueryArtifactDeletionPosture::DeleteWithRun,
        domain::WorthQueryArtifactLegalHoldPosture::NotEligible,
    ))
    .compatibility(domain::WorthQueryArtifactCompatibilityContract::new(
        domain::WorthQueryArtifactCompatibilityWindow::new(
            domain::WorthQueryArtifactSchemaVersion::new(1),
            domain::WorthQueryArtifactSchemaVersion::new(1),
            domain::WorthQueryArtifactProtocolVersion::new(1),
            domain::WorthQueryArtifactProtocolVersion::new(1),
        ),
        "WORTH.tests.artifact-workflow.migration",
        domain::WorthQueryArtifactRetirementRule::Active,
        domain::WorthQueryArtifactDowngradePosture::Denied,
    ))
    .produced_by(["produce", "consume"])
    .consumed_by(["consume", "observe-a", "observe-b"])
    .finish()
    .unwrap()
}

pub fn artifact_support() -> domain::WorthQueryArtifactInstallationSupport {
    domain::WorthQueryArtifactInstallationSupport::new()
        .artifact_version::<CandidateArtifactFamily>(
            domain::WorthQueryArtifactSchemaVersion::new(1),
            domain::WorthQueryArtifactProtocolVersion::new(1),
            domain::WorthQueryArtifactVersionSupport::Admitted,
        )
}

pub fn candidate_layout_contract() -> domain::WorthQueryArtifactNativeLayoutContract {
    domain::WorthQueryArtifactNativeLayoutContract::new(
        domain::WorthQueryArtifactNativeLayoutIdentity::new("candidate-columns"),
        domain::WorthQueryArtifactNativeLayoutVersion::new(1),
        domain::WorthQueryArtifactNativeAlignment::new(8),
        [
            domain::WorthQueryArtifactNativeFieldContract::new(
                candidate_id_contract(),
                domain::WorthQueryArtifactFieldSlicePosture::Borrowed,
            ),
            domain::WorthQueryArtifactNativeFieldContract::new(
                candidate_score_contract(),
                domain::WorthQueryArtifactFieldSlicePosture::Borrowed,
            ),
            domain::WorthQueryArtifactNativeFieldContract::new(
                candidate_token_contract(),
                domain::WorthQueryArtifactFieldSlicePosture::ProviderNativeProjectionOnly,
            ),
            domain::WorthQueryArtifactNativeFieldContract::new(
                candidate_target_contract(),
                domain::WorthQueryArtifactFieldSlicePosture::ProviderNativeProjectionOnly,
            ),
            domain::WorthQueryArtifactNativeFieldContract::new(
                candidate_content_contract(),
                domain::WorthQueryArtifactFieldSlicePosture::ProviderNativeProjectionOnly,
            ),
        ],
    )
}

pub fn candidate_layout() -> domain::WorthQueryArtifactNativeLayoutReference {
    candidate_layout_contract().reference()
}

pub fn foreign_candidate_layout() -> domain::WorthQueryArtifactNativeLayoutReference {
    domain::WorthQueryArtifactNativeLayoutContract::new(
        domain::WorthQueryArtifactNativeLayoutIdentity::new("foreign-candidate-columns"),
        domain::WorthQueryArtifactNativeLayoutVersion::new(1),
        domain::WorthQueryArtifactNativeAlignment::new(8),
        [domain::WorthQueryArtifactNativeFieldContract::new(
            candidate_id_contract(),
            domain::WorthQueryArtifactFieldSlicePosture::Borrowed,
        )],
    )
    .reference()
}

pub fn misaligned_candidate_layout() -> domain::WorthQueryArtifactNativeLayoutReference {
    domain::WorthQueryArtifactNativeLayoutContract::new(
        domain::WorthQueryArtifactNativeLayoutIdentity::new("candidate-columns"),
        domain::WorthQueryArtifactNativeLayoutVersion::new(1),
        domain::WorthQueryArtifactNativeAlignment::new(4),
        [domain::WorthQueryArtifactNativeFieldContract::new(
            candidate_id_contract(),
            domain::WorthQueryArtifactFieldSlicePosture::Borrowed,
        )],
    )
    .reference()
}

pub fn candidate_id() -> AspectKey {
    AspectKey::new("candidate.id").unwrap()
}

pub fn candidate_score() -> AspectKey {
    AspectKey::new("candidate.score").unwrap()
}

pub fn candidate_token() -> AspectKey {
    AspectKey::new("candidate.token").unwrap()
}

pub fn candidate_target() -> AspectKey {
    AspectKey::new("candidate.target").unwrap()
}

pub fn candidate_content() -> AspectKey {
    AspectKey::new("candidate.content").unwrap()
}

pub fn candidate_id_contract() -> AspectContract {
    scalar_contract(candidate_id(), 0x9150_0001, ScalarAspectType::UInt64)
}

pub fn candidate_score_contract() -> AspectContract {
    scalar_contract(candidate_score(), 0x9150_0002, ScalarAspectType::Float64)
}

pub fn candidate_signature_contract() -> AspectContract {
    scalar_contract(
        AspectKey::new("candidate.signature").unwrap(),
        0x9150_0006,
        ScalarAspectType::UInt64,
    )
}

pub fn candidate_token_contract() -> AspectContract {
    AspectContract::opaque_token(
        candidate_token(),
        AspectIdentity(0x9150_0003),
        AspectContractRevision(1),
    )
}

pub fn candidate_target_contract() -> AspectContract {
    AspectContract::reference_entity(
        candidate_target(),
        AspectIdentity(0x9150_0004),
        AspectContractRevision(1),
    )
}

pub fn candidate_content_contract() -> AspectContract {
    AspectContract::content_ref(
        candidate_content(),
        AspectIdentity(0x9150_0005),
        AspectContractRevision(1),
    )
}

pub fn scalar_contract(key: AspectKey, identity: u64, family: ScalarAspectType) -> AspectContract {
    AspectContract::scalar(
        key,
        AspectIdentity(identity),
        AspectContractRevision(1),
        family,
    )
}

pub fn counter(name: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(name).unwrap()
}
