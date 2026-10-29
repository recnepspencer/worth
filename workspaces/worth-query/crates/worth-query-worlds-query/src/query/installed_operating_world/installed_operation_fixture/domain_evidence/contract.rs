pub use worth_foundational::facade::{
    CanonicalizationRuleVersion, FoundationalPerformanceCounterName, RetentionDeliveryProfile,
};
pub use worth_query::facade::domain;

pub use super::super::{canonical_bundle, semantic_closure, GeometryDomain};
pub use super::graph_receipt::EvidenceGraph;
pub use super::{EvidenceFamily, EvidenceRead};

pub struct EvidenceArtifactFamily;

#[derive(Clone, Copy)]
pub struct EvidenceGovernance {
    pub redaction: domain::WorthQueryArtifactRedactionPosture,
    pub retention: RetentionDeliveryProfile,
    pub decision_retention: RetentionDeliveryProfile,
    pub secondary_decision_retention: Option<RetentionDeliveryProfile>,
    pub deletion: domain::WorthQueryArtifactDeletionPosture,
    pub legal_hold: domain::WorthQueryArtifactLegalHoldPosture,
}

impl EvidenceGovernance {
    pub fn retained(redaction: domain::WorthQueryArtifactRedactionPosture) -> Self {
        Self {
            redaction,
            retention: RetentionDeliveryProfile::Durable,
            decision_retention: RetentionDeliveryProfile::Durable,
            secondary_decision_retention: None,
            deletion: domain::WorthQueryArtifactDeletionPosture::DomainControlled,
            legal_hold: domain::WorthQueryArtifactLegalHoldPosture::RequiredWhenDirected,
        }
    }
}

impl domain::WorthQueryArtifactFamily for EvidenceArtifactFamily {
    const SEMANTIC_FAMILY: &'static str = "WORTH.tests.domain-evidence";
}

pub fn direct_package(
    redaction: domain::WorthQueryArtifactRedactionPosture,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    direct_package_with_graph(EvidenceGovernance::retained(redaction), false)
}

pub fn direct_graph_package(
    redaction: domain::WorthQueryArtifactRedactionPosture,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    direct_package_with_graph(EvidenceGovernance::retained(redaction), true)
}

pub fn direct_package_with_governance(
    governance: EvidenceGovernance,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    direct_package_with_graph(governance, false)
}

pub fn direct_package_with_graph(
    governance: EvidenceGovernance,
    graph: bool,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let contract = evidence_contract(governance);
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.evidence =
        domain::WorthQueryDomainEvidenceContract::installed_artifact(contract.reference());
    semantics.lowering = domain::WorthQueryOperationLoweringContract {
        family: "domain-evidence-read-v1".into(),
        deterministic: true,
    };
    if graph {
        add_evidence_graph_read(&mut semantics.graph_reads);
        semantics.cost.execution = domain::WorthQueryOperationCostClass::ExternalBoundary;
    }
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        EvidenceRead,
        EvidenceFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("evidence-read", 1),
        semantics,
    );
    let package = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
    .artifact_contract(contract);
    if graph {
        package.operation_graph_participation::<EvidenceRead, EvidenceFamily, EvidenceGraph>(
            "evidence-graph",
        )
    } else {
        package
    }
}

pub fn add_evidence_graph_read(reads: &mut domain::WorthQueryOperationGraphReadContract) {
    match reads {
        domain::WorthQueryOperationGraphReadContract::DeclaredDomain { roles } => {
            roles.push(domain::WorthQueryDomainOperationGraphReadRole {
                role: "evidence-graph".into(),
                participation: domain::WorthQueryOperationGraphParticipation::SeparateAuthority {
                    role: "evidence-graph".into(),
                },
                access: domain::WorthQueryOperationGraphAccess::Observe,
                semantic_reads: Vec::new(),
            });
        }
        domain::WorthQueryOperationGraphReadContract::NotRequired => {
            unreachable!("evidence operations retain the primary graph read")
        }
        domain::WorthQueryOperationGraphReadContract::Declared { .. } => {
            unreachable!("portable domain operations cannot carry application read scopes")
        }
    }
}

pub fn artifact_support() -> domain::WorthQueryArtifactInstallationSupport {
    domain::WorthQueryArtifactInstallationSupport::new().artifact_version::<EvidenceArtifactFamily>(
        domain::WorthQueryArtifactSchemaVersion::new(1),
        domain::WorthQueryArtifactProtocolVersion::new(1),
        domain::WorthQueryArtifactVersionSupport::Admitted,
    )
}

pub fn evidence_contract(
    governance: EvidenceGovernance,
) -> domain::WorthQueryPortableArtifactContract {
    domain::WorthQueryPortableArtifactContract::declare::<EvidenceArtifactFamily>(
        domain::WorthQueryArtifactSchemaVersion::new(1),
        domain::WorthQueryArtifactProtocolVersion::new(1),
    )
    .identity(
        domain::WorthQueryArtifactContentIdentityContract::owner_canonical_projection(
            "WORTH.tests.domain-evidence.projection",
            CanonicalizationRuleVersion::new("domain-evidence-v1").unwrap(),
        ),
    )
    .ownership(domain::WorthQueryArtifactOwnershipContract::domain_payload(
        "WORTH.tests.domain-evidence",
        "WORTH.tests.domain-evidence.provider",
    ))
    .occurrence(domain::WorthQueryArtifactOccurrenceContract::independent_per_execution())
    .evidence(domain::WorthQueryArtifactEvidenceContract::new(
        "domain-evidence-basis",
        "domain-evidence-provenance",
        "domain-evidence-dependency",
        "domain-evidence-invalidation",
        "domain-evidence-equivalence",
    ))
    .reproducibility(domain::WorthQueryArtifactReproducibilityContract::new(
        domain::WorthQueryArtifactReproducibilityClass::ExactDeterministic,
        domain::WorthQueryArtifactDeterminismPosture::Deterministic,
        domain::WorthQueryArtifactComparisonAuthority::ExactCanonicalValue,
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    ))
    .search(domain::WorthQueryCandidateSearchContract::declared(
        domain::WorthQueryCandidateSearchEvidenceFamilies::new(
            "candidate-universe",
            "candidate-termination",
            "candidate-feasibility",
            "candidate-comparison",
            "candidate-incumbent",
        ),
        domain::WorthQueryCandidateSearchPosture::Sampled {
            sample_identity: "sample-v1".into(),
        },
        domain::WorthQueryCandidateOptimalityPosture::BestInDeclaredSample {
            sample_identity: "sample-v1".into(),
        },
    ))
    .convergence(domain::WorthQueryConvergenceContract::NotIterative)
    .transformation(domain::WorthQueryTransformationEvidenceContract::declared(
        domain::WorthQueryImmutableSourceOccurrenceContract::new("source-occurrence"),
        domain::WorthQueryTransformationIdentity::new("normalize-candidates", 1),
        domain::WorthQueryTransformationOutcomeContract::new(
            domain::WorthQuerySourceOutputCorrespondence::OneToMany,
            domain::WorthQueryTransformationDisposition::Normalized,
            domain::WorthQueryTransformationErrorPosture::Bounded,
            domain::WorthQueryTransformationLossPosture::DeclaredLossy,
        ),
    ))
    .access_path(domain::WorthQueryArtifactAccessPathContract::denied())
    .carriage(domain::WorthQueryArtifactCarriageContract::move_only_provider_transfer())
    .lifecycle(domain::WorthQueryArtifactLifecycleContract::Retained)
    .counters(counter_contract())
    .decisions(decision_contract(
        governance.decision_retention,
        governance.secondary_decision_retention,
    ))
    .governance(domain::WorthQueryArtifactGovernanceContract::new(
        ["audit", "support"],
        domain::WorthQueryArtifactClassification::Restricted,
        governance.redaction,
        governance.retention,
        governance.deletion,
        governance.legal_hold,
    ))
    .compatibility(domain::WorthQueryArtifactCompatibilityContract::new(
        domain::WorthQueryArtifactCompatibilityWindow::new(
            domain::WorthQueryArtifactSchemaVersion::new(1),
            domain::WorthQueryArtifactSchemaVersion::new(1),
            domain::WorthQueryArtifactProtocolVersion::new(1),
            domain::WorthQueryArtifactProtocolVersion::new(1),
        ),
        "WORTH.tests.domain-evidence.migration",
        domain::WorthQueryArtifactRetirementRule::Active,
        domain::WorthQueryArtifactDowngradePosture::Denied,
    ))
    .produced_by(["evidence-read:1", "evidence", "left", "start"])
    .consumed_by(["audit"])
    .finish()
    .unwrap()
}

pub fn counter_contract() -> domain::WorthQueryStructuralCounterContract {
    domain::WorthQueryStructuralCounterContract::declare([
        counter_schema(
            "bytes",
            domain::WorthQueryStructuralCounterRole::Bytes,
            domain::WorthQueryStructuralCounterUnit::Bytes,
            domain::WorthQueryStructuralCounterAggregation::Independent,
            domain::WorthQueryStructuralCounterScope::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterReplayPosture::Exact,
        ),
        counter_schema(
            "elements",
            domain::WorthQueryStructuralCounterRole::Elements,
            domain::WorthQueryStructuralCounterUnit::Elements,
            domain::WorthQueryStructuralCounterAggregation::Independent,
            domain::WorthQueryStructuralCounterScope::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterReplayPosture::Exact,
        ),
        counter_schema(
            "work",
            domain::WorthQueryStructuralCounterRole::StructuralWork,
            domain::WorthQueryStructuralCounterUnit::Operations,
            domain::WorthQueryStructuralCounterAggregation::SumOf(vec![counter(
                "operation-work-components",
            )]),
            domain::WorthQueryStructuralCounterScope::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterReplayPosture::Exact,
        ),
        counter_schema(
            "operation-work-components",
            domain::WorthQueryStructuralCounterRole::DomainWork,
            domain::WorthQueryStructuralCounterUnit::Operations,
            domain::WorthQueryStructuralCounterAggregation::Independent,
            domain::WorthQueryStructuralCounterScope::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
            domain::WorthQueryStructuralCounterReplayPosture::Exact,
        ),
        counter_schema(
            "candidate-comparisons",
            domain::WorthQueryStructuralCounterRole::DomainWork,
            domain::WorthQueryStructuralCounterUnit::Comparisons,
            domain::WorthQueryStructuralCounterAggregation::Independent,
            domain::WorthQueryStructuralCounterScope::Operation,
            domain::WorthQueryStructuralCounterResetBoundary::Operation,
            domain::WorthQueryStructuralCounterReplayPosture::NonDecreasing,
        ),
        optional_counter_schema(),
    ])
}

pub fn counter_schema(
    name: &str,
    role: domain::WorthQueryStructuralCounterRole,
    unit: domain::WorthQueryStructuralCounterUnit,
    aggregation: domain::WorthQueryStructuralCounterAggregation,
    scope: domain::WorthQueryStructuralCounterScope,
    reset: domain::WorthQueryStructuralCounterResetBoundary,
    replay: domain::WorthQueryStructuralCounterReplayPosture,
) -> domain::WorthQueryStructuralCounterSchema {
    domain::WorthQueryStructuralCounterSchema::new(
        counter(name),
        role,
        unit,
        aggregation,
        domain::WorthQueryStructuralCounterMonotonicity::NonDecreasing,
        scope,
        reset,
        domain::WorthQueryStructuralCounterRequiredness::RequiredCore,
        replay,
    )
}

pub fn optional_counter_schema() -> domain::WorthQueryStructuralCounterSchema {
    domain::WorthQueryStructuralCounterSchema::new(
        counter("trace-events"),
        domain::WorthQueryStructuralCounterRole::DomainWork,
        domain::WorthQueryStructuralCounterUnit::Iterations,
        domain::WorthQueryStructuralCounterAggregation::Independent,
        domain::WorthQueryStructuralCounterMonotonicity::NonDecreasing,
        domain::WorthQueryStructuralCounterScope::ArtifactOccurrence,
        domain::WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
        domain::WorthQueryStructuralCounterRequiredness::OptionalSidecar,
        domain::WorthQueryStructuralCounterReplayPosture::ProviderCertified,
    )
}

pub fn decision_contract(
    retention: RetentionDeliveryProfile,
    secondary_retention: Option<RetentionDeliveryProfile>,
) -> domain::WorthQueryDecisionRecordContract {
    let mut schemas = vec![domain::WorthQueryDecisionSchema::new(
        domain::WorthQueryDecisionIdentity::new(
            domain::WorthQueryDecisionKind::new("candidate-rejected").unwrap(),
            domain::WorthQueryDecisionReasonFamily::new("search-reason").unwrap(),
            domain::WorthQueryArtifactKeyFamily::new("candidate").unwrap(),
        ),
        domain::WorthQueryDecisionCausalParentShape::RequiredSingle,
        domain::WorthQueryDecisionPayloadVersion::new(1),
        domain::WorthQueryDecisionGovernance::new(
            domain::WorthQueryArtifactClassification::Restricted,
            retention,
        ),
    )];
    if let Some(retention) = secondary_retention {
        schemas.push(domain::WorthQueryDecisionSchema::new(
            domain::WorthQueryDecisionIdentity::new(
                domain::WorthQueryDecisionKind::new("candidate-selected").unwrap(),
                domain::WorthQueryDecisionReasonFamily::new("selection-reason").unwrap(),
                domain::WorthQueryArtifactKeyFamily::new("candidate").unwrap(),
            ),
            domain::WorthQueryDecisionCausalParentShape::RequiredSingle,
            domain::WorthQueryDecisionPayloadVersion::new(1),
            domain::WorthQueryDecisionGovernance::new(
                domain::WorthQueryArtifactClassification::Restricted,
                retention,
            ),
        ));
    }
    domain::WorthQueryDecisionRecordContract::declared(schemas)
}

pub fn counter(name: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(name).unwrap()
}
