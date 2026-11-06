use worth_foundational::facade::CanonicalizationRuleVersion;
use worth_foundational::facade::FoundationalPerformanceCounterName;
use worth_foundational::facade::RetentionDeliveryProfile;
use worth_query_installation::facade::WorthQueryArtifactBorrowPosture;
use worth_query_installation::facade::WorthQueryArtifactCarriageContract;
use worth_query_installation::facade::WorthQueryArtifactClassification;
use worth_query_installation::facade::WorthQueryArtifactClonePosture;
use worth_query_installation::facade::WorthQueryArtifactComparatorFamily;
use worth_query_installation::facade::WorthQueryArtifactCompatibilityContract;
use worth_query_installation::facade::WorthQueryArtifactCompatibilityWindow;
use worth_query_installation::facade::WorthQueryArtifactContentIdentityContract;
use worth_query_installation::facade::WorthQueryArtifactDeletionPosture;
use worth_query_installation::facade::WorthQueryArtifactDowngradePosture;
use worth_query_installation::facade::WorthQueryArtifactEvidenceContract;
use worth_query_installation::facade::WorthQueryArtifactGovernanceContract;
use worth_query_installation::facade::WorthQueryArtifactLegalHoldPosture;
use worth_query_installation::facade::WorthQueryArtifactLifecycleContract;
use worth_query_installation::facade::WorthQueryArtifactMovePosture;
use worth_query_installation::facade::WorthQueryArtifactOccurrenceContract;
use worth_query_installation::facade::WorthQueryArtifactOwnershipContract;
use worth_query_installation::facade::WorthQueryArtifactProtocolVersion;
use worth_query_installation::facade::WorthQueryArtifactProviderTransferPosture;
use worth_query_installation::facade::WorthQueryArtifactRedactionPosture;
use worth_query_installation::facade::WorthQueryArtifactReproducibilityContract;
use worth_query_installation::facade::WorthQueryArtifactRetirementRule;
use worth_query_installation::facade::WorthQueryArtifactSchemaVersion;
use worth_query_installation::facade::WorthQueryArtifactSerializationPosture;
use worth_query_installation::facade::WorthQueryCandidateOptimalityPosture;
use worth_query_installation::facade::WorthQueryCandidateSearchContract;
use worth_query_installation::facade::WorthQueryCandidateSearchEvidenceFamilies;
use worth_query_installation::facade::WorthQueryCandidateSearchPosture;
use worth_query_installation::facade::WorthQueryConvergenceContract;
use worth_query_installation::facade::WorthQueryConvergenceIncumbentPosture;
use worth_query_installation::facade::WorthQueryConvergenceOscillationPosture;
use worth_query_installation::facade::WorthQueryDecisionRecordContract;
use worth_query_installation::facade::WorthQueryPortableArtifactContract;
use worth_query_installation::facade::WorthQueryStructuralCounterContract;
use worth_query_installation::facade::WorthQueryTransformationEvidenceContract;

use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::CandidateFamily;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::ComparatorFamily;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::OWNER;

#[derive(Clone, Copy)]
pub enum FixtureConvergenceContract {
    Bounded,
    Pareto,
    OscillationImpossible,
    OscillationSelectIncumbent,
    OscillationDomainClassified,
    NonIterative,
    MissingSearch,
}

pub fn candidate_contract(
    producer: &str,
    fixture: FixtureConvergenceContract,
) -> WorthQueryPortableArtifactContract {
    let search = match fixture {
        FixtureConvergenceContract::MissingSearch => {
            WorthQueryCandidateSearchContract::not_applicable()
        }
        FixtureConvergenceContract::Bounded
        | FixtureConvergenceContract::Pareto
        | FixtureConvergenceContract::OscillationImpossible
        | FixtureConvergenceContract::OscillationSelectIncumbent
        | FixtureConvergenceContract::OscillationDomainClassified
        | FixtureConvergenceContract::NonIterative => WorthQueryCandidateSearchContract::declared(
            WorthQueryCandidateSearchEvidenceFamilies::new(
                "worth.convergence.universe",
                "worth.convergence.termination",
                "worth.convergence.feasibility",
                "worth.convergence.comparison",
                "worth.convergence.incumbent",
            ),
            WorthQueryCandidateSearchPosture::Heuristic,
            WorthQueryCandidateOptimalityPosture::Unknown,
        ),
    };
    let convergence = match fixture {
        FixtureConvergenceContract::NonIterative => WorthQueryConvergenceContract::NotIterative,
        FixtureConvergenceContract::Pareto => WorthQueryConvergenceContract::Iterative {
            progress_measure_family: "worth.convergence.progress".into(),
            comparator_family: ComparatorFamily::SEMANTIC_FAMILY.into(),
            repeated_state_family: "worth.convergence.repeated-state".into(),
            incumbent: WorthQueryConvergenceIncumbentPosture::ParetoFrontier,
            iteration_bound: 3,
            oscillation: WorthQueryConvergenceOscillationPosture::DetectAndDeny,
        },
        FixtureConvergenceContract::OscillationImpossible => {
            iterative_contract(WorthQueryConvergenceOscillationPosture::Impossible)
        }
        FixtureConvergenceContract::OscillationSelectIncumbent => {
            iterative_contract(WorthQueryConvergenceOscillationPosture::DetectAndSelectIncumbent)
        }
        FixtureConvergenceContract::OscillationDomainClassified => {
            iterative_contract(WorthQueryConvergenceOscillationPosture::DomainClassified)
        }
        FixtureConvergenceContract::Bounded | FixtureConvergenceContract::MissingSearch => {
            WorthQueryConvergenceContract::bounded(
                "worth.convergence.progress",
                ComparatorFamily::SEMANTIC_FAMILY,
                "worth.convergence.repeated-state",
                3,
            )
        }
    };
    WorthQueryPortableArtifactContract::declare::<CandidateFamily>(
        WorthQueryArtifactSchemaVersion::new(1),
        WorthQueryArtifactProtocolVersion::new(1),
    )
    .identity(
        WorthQueryArtifactContentIdentityContract::owner_canonical_projection(
            "worth.convergence.candidate-projection",
            CanonicalizationRuleVersion::new("convergence-candidate-v1").unwrap(),
        ),
    )
    .ownership(WorthQueryArtifactOwnershipContract::domain_payload(
        OWNER,
        "worth.convergence.provider",
    ))
    .occurrence(WorthQueryArtifactOccurrenceContract::independent_per_execution())
    .evidence(WorthQueryArtifactEvidenceContract::new(
        "worth.convergence.basis",
        "worth.convergence.provenance",
        "worth.convergence.dependency",
        "worth.convergence.invalidation",
        "worth.convergence.equivalence",
    ))
    .reproducibility(
        WorthQueryArtifactReproducibilityContract::domain_comparator(
            ComparatorFamily::SEMANTIC_FAMILY,
            ["fixture-model"],
        ),
    )
    .search(search)
    .convergence(convergence)
    .transformation(WorthQueryTransformationEvidenceContract::not_a_transformation())
    .access_path(worth_query_installation::facade::WorthQueryArtifactAccessPathContract::denied())
    .carriage(WorthQueryArtifactCarriageContract::new(
        WorthQueryArtifactMovePosture::Required,
        WorthQueryArtifactBorrowPosture::SharedReadOnly,
        WorthQueryArtifactClonePosture::Forbidden,
        WorthQueryArtifactProviderTransferPosture::MoveOwnership,
        WorthQueryArtifactSerializationPosture::CanonicalProjectionOnly,
    ))
    .lifecycle(WorthQueryArtifactLifecycleContract::ArenaScoped)
    .counters(WorthQueryStructuralCounterContract::required_foundation(
        counter("candidate-bytes"),
        counter("candidate-elements"),
        counter("comparison-work"),
    ))
    .decisions(WorthQueryDecisionRecordContract::not_required())
    .governance(WorthQueryArtifactGovernanceContract::new(
        ["workflow-internal"],
        WorthQueryArtifactClassification::Internal,
        WorthQueryArtifactRedactionPosture::NotRequired,
        RetentionDeliveryProfile::Ephemeral,
        WorthQueryArtifactDeletionPosture::DeleteWithRun,
        WorthQueryArtifactLegalHoldPosture::NotEligible,
    ))
    .compatibility(WorthQueryArtifactCompatibilityContract::new(
        WorthQueryArtifactCompatibilityWindow::new(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
        ),
        "worth.convergence.migration",
        WorthQueryArtifactRetirementRule::Active,
        WorthQueryArtifactDowngradePosture::Denied,
    ))
    .produced_by([producer])
    .consumed_by(["convergence-runtime"])
    .finish()
    .expect("convergence fixture artifact must validate")
}

pub fn iterative_contract(
    oscillation: WorthQueryConvergenceOscillationPosture,
) -> WorthQueryConvergenceContract {
    WorthQueryConvergenceContract::Iterative {
        progress_measure_family: "worth.convergence.progress".into(),
        comparator_family: ComparatorFamily::SEMANTIC_FAMILY.into(),
        repeated_state_family: "worth.convergence.repeated-state".into(),
        incumbent: WorthQueryConvergenceIncumbentPosture::BestObserved,
        iteration_bound: 3,
        oscillation,
    }
}

pub fn counter(name: &str) -> FoundationalPerformanceCounterName {
    FoundationalPerformanceCounterName::new(name).unwrap()
}
