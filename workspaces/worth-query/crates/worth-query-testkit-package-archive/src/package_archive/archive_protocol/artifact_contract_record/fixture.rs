pub use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, CanonicalizationRuleVersion,
    FoundationalPerformanceCounterName, RetentionDeliveryProfile, ScalarAspectType,
};
pub use worth_query_installation::facade::*;

pub struct ArchiveArtifactFamily;

impl WorthQueryArtifactFamily for ArchiveArtifactFamily {
    const SEMANTIC_FAMILY: &'static str = "worth.archive.candidates";
}

pub fn artifact_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.artifact",
        1,
        0,
    ))
    .artifact_contract(artifact_contract())
    .validate()
    .unwrap()
}

pub fn artifact_contract() -> WorthQueryPortableArtifactContract {
    WorthQueryPortableArtifactContract::declare::<ArchiveArtifactFamily>(
        WorthQueryArtifactSchemaVersion::new(2),
        WorthQueryArtifactProtocolVersion::new(1),
    )
    .identity(
        WorthQueryArtifactContentIdentityContract::owner_canonical_projection(
            "worth.archive.candidate-projection",
            CanonicalizationRuleVersion::new("candidate-v2").unwrap(),
        ),
    )
    .ownership(WorthQueryArtifactOwnershipContract::domain_payload(
        "worth.archive",
        "worth.archive.candidate-provider",
    ))
    .occurrence(
        WorthQueryArtifactOccurrenceContract::independent_per_execution()
            .permit(WorthQueryArtifactSubstitutionPurpose::ComputationalReuse)
            .permit(WorthQueryArtifactSubstitutionPurpose::EvidentiarySubstitution),
    )
    .evidence(WorthQueryArtifactEvidenceContract::new(
        "worth.archive.basis",
        "worth.archive.provenance",
        "worth.archive.dependency",
        "worth.archive.invalidation",
        "worth.archive.equivalence",
    ))
    .reproducibility(WorthQueryArtifactReproducibilityContract::new(
        WorthQueryArtifactReproducibilityClass::DomainComparator,
        WorthQueryArtifactDeterminismPosture::EnvironmentDependent,
        WorthQueryArtifactComparisonAuthority::RegisteredDomainComparator {
            family: "worth.archive.comparator".into(),
        },
        ["model-v2", "solver-v4"],
        ["seed-source"],
    ))
    .search(WorthQueryCandidateSearchContract::declared(
        WorthQueryCandidateSearchEvidenceFamilies::new(
            "worth.archive.universe",
            "worth.archive.termination",
            "worth.archive.feasibility",
            "worth.archive.comparison",
            "worth.archive.incumbent",
        ),
        WorthQueryCandidateSearchPosture::ProvenTopK { count: 10 },
        WorthQueryCandidateOptimalityPosture::ProvenTopK { count: 10 },
    ))
    .convergence(WorthQueryConvergenceContract::Iterative {
        progress_measure_family: "worth.archive.progress".into(),
        comparator_family: "worth.archive.convergence".into(),
        repeated_state_family: "worth.archive.repeated-state".into(),
        incumbent: WorthQueryConvergenceIncumbentPosture::ParetoFrontier,
        iteration_bound: 100,
        oscillation: WorthQueryConvergenceOscillationPosture::DetectAndSelectIncumbent,
    })
    .transformation(WorthQueryTransformationEvidenceContract::declared(
        WorthQueryImmutableSourceOccurrenceContract::new("worth.archive.source-occurrence"),
        WorthQueryTransformationIdentity::new("worth.archive.normalization", 3),
        WorthQueryTransformationOutcomeContract::new(
            WorthQuerySourceOutputCorrespondence::ManyToOne,
            WorthQueryTransformationDisposition::Normalized,
            WorthQueryTransformationErrorPosture::Bounded,
            WorthQueryTransformationLossPosture::DeclaredLossy,
        ),
    ))
    .access_path(native_access())
    .carriage(WorthQueryArtifactCarriageContract::new(
        WorthQueryArtifactMovePosture::Required,
        WorthQueryArtifactBorrowPosture::SharedReadOnly,
        WorthQueryArtifactClonePosture::Declared {
            mechanism: WorthQueryArtifactCloneMechanism::DeepClone,
            boundary: WorthQueryArtifactCloneBoundary::Isolation,
        },
        WorthQueryArtifactProviderTransferPosture::MoveOwnership,
        WorthQueryArtifactSerializationPosture::CanonicalProjectionOnly,
    ))
    .lifecycle(WorthQueryArtifactLifecycleContract::Retained)
    .counters(counters())
    .decisions(decisions())
    .governance(WorthQueryArtifactGovernanceContract::new(
        ["audit", "workflow-internal"],
        WorthQueryArtifactClassification::Restricted,
        WorthQueryArtifactRedactionPosture::DomainRedactorRequired,
        RetentionDeliveryProfile::Durable,
        WorthQueryArtifactDeletionPosture::DomainControlled,
        WorthQueryArtifactLegalHoldPosture::RequiredWhenDirected,
    ))
    .compatibility(WorthQueryArtifactCompatibilityContract::new(
        WorthQueryArtifactCompatibilityWindow::new(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactSchemaVersion::new(3),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(2),
        ),
        "worth.archive.migration",
        WorthQueryArtifactRetirementRule::RetiredThroughSchema(
            WorthQueryArtifactSchemaVersion::new(1),
        ),
        WorthQueryArtifactDowngradePosture::SupportedBy {
            family: "worth.archive.downgrade".into(),
        },
    ))
    .produced_by(["collect", "normalize"])
    .consumed_by(["publish", "rank"])
    .finish()
    .unwrap()
}

pub fn native_access() -> WorthQueryArtifactAccessPathContract {
    WorthQueryArtifactAccessPathContract::native(WorthQueryArtifactNativeAccessContract::new(
        WorthQueryArtifactNativeLayoutContract::new(
            WorthQueryArtifactNativeLayoutIdentity::new("candidate-columns"),
            WorthQueryArtifactNativeLayoutVersion::new(2),
            WorthQueryArtifactNativeAlignment::new(8),
            [
                WorthQueryArtifactNativeFieldContract::new(
                    scalar("candidate.id", 1, ScalarAspectType::UInt64),
                    WorthQueryArtifactFieldSlicePosture::Borrowed,
                ),
                WorthQueryArtifactNativeFieldContract::new(
                    scalar("candidate.score", 2, ScalarAspectType::Float64),
                    WorthQueryArtifactFieldSlicePosture::Borrowed,
                ),
            ],
        ),
        WorthQueryArtifactRowBatchPosture::Borrowed,
        Some(WorthQueryArtifactChunkContract::bounded(64)),
        [WorthQueryArtifactBulkProjectionContract::new(
            "candidate-summary",
            [aspect_key("candidate.id"), aspect_key("candidate.score")],
            WorthQueryArtifactNativeAlignment::new(8),
            [scalar("candidate.summary", 3, ScalarAspectType::Float64)],
        )],
        WorthQueryArtifactScalarFallbackPosture::admitted(16, 4),
    ))
}

pub fn counters() -> WorthQueryStructuralCounterContract {
    WorthQueryStructuralCounterContract::declare([
        counter_schema(
            "artifact-bytes",
            WorthQueryStructuralCounterRole::Bytes,
            WorthQueryStructuralCounterUnit::Bytes,
            WorthQueryStructuralCounterAggregation::Independent,
            WorthQueryStructuralCounterRequiredness::RequiredCore,
        ),
        counter_schema(
            "artifact-elements",
            WorthQueryStructuralCounterRole::Elements,
            WorthQueryStructuralCounterUnit::Elements,
            WorthQueryStructuralCounterAggregation::Independent,
            WorthQueryStructuralCounterRequiredness::RequiredCore,
        ),
        counter_schema(
            "artifact-work",
            WorthQueryStructuralCounterRole::StructuralWork,
            WorthQueryStructuralCounterUnit::Operations,
            WorthQueryStructuralCounterAggregation::Independent,
            WorthQueryStructuralCounterRequiredness::RequiredCore,
        ),
        counter_schema(
            "total-bytes",
            WorthQueryStructuralCounterRole::DomainWork,
            WorthQueryStructuralCounterUnit::Bytes,
            WorthQueryStructuralCounterAggregation::SumOf(vec![counter("artifact-bytes")]),
            WorthQueryStructuralCounterRequiredness::OptionalSidecar,
        ),
    ])
}

pub fn counter_schema(
    name: &str,
    role: WorthQueryStructuralCounterRole,
    unit: WorthQueryStructuralCounterUnit,
    aggregation: WorthQueryStructuralCounterAggregation,
    requiredness: WorthQueryStructuralCounterRequiredness,
) -> WorthQueryStructuralCounterSchema {
    WorthQueryStructuralCounterSchema::new(
        counter(name),
        role,
        unit,
        aggregation,
        WorthQueryStructuralCounterMonotonicity::NonDecreasing,
        WorthQueryStructuralCounterScope::ArtifactOccurrence,
        WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
        requiredness,
        WorthQueryStructuralCounterReplayPosture::Exact,
    )
}

pub fn decisions() -> WorthQueryDecisionRecordContract {
    WorthQueryDecisionRecordContract::declared([WorthQueryDecisionSchema::new(
        WorthQueryDecisionIdentity::new(
            WorthQueryDecisionKind::new("candidate-selected").unwrap(),
            WorthQueryDecisionReasonFamily::new("ranking-result").unwrap(),
            WorthQueryArtifactKeyFamily::new("candidate-key").unwrap(),
        ),
        WorthQueryDecisionCausalParentShape::OrderedMany,
        WorthQueryDecisionPayloadVersion::new(2),
        WorthQueryDecisionGovernance::new(
            WorthQueryArtifactClassification::Confidential,
            RetentionDeliveryProfile::Retained,
        ),
    )])
}

pub fn scalar(key: &str, identity: u64, family: ScalarAspectType) -> AspectContract {
    AspectContract::scalar(
        aspect_key(key),
        AspectIdentity(identity),
        AspectContractRevision(1),
        family,
    )
}

pub fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).unwrap()
}

pub fn counter(value: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(value).unwrap()
}
