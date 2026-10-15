use super::{
    IdentityEvolutionCertificationFailureClass, IdentityEvolutionCertificationPerturbationClass,
};
use crate::harness::certification::HostileExpectation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityEvolutionCanonicalRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: IdentityEvolutionCertificationPerturbationClass,
    pub hostile_expectation: HostileExpectation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityEvolutionRejectionRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: IdentityEvolutionCertificationPerturbationClass,
    pub failure_class: IdentityEvolutionCertificationFailureClass,
}

pub const IDENTITY_EVOLUTION_CANONICAL_ROW_SPECS: &[IdentityEvolutionCanonicalRowSpec] = &[
    IdentityEvolutionCanonicalRowSpec {
        row_name: "replacement-continuity-explicitness",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::LineageTraversal,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "split-successor-explicitness",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::LineageTraversal,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "branch-local-divergence-explicitness",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::BranchLocality,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "ambiguous-correspondence-explicitness",
        perturbation_class:
            IdentityEvolutionCertificationPerturbationClass::CorrespondenceComparison,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "identity-break-explicitness",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::IdentityBreak,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "identity-aware-inspector-consumption-parity",
        perturbation_class:
            IdentityEvolutionCertificationPerturbationClass::CrossFeatureConsumption,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "lineage-versus-structural-disagreement-explicitness",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::Disagreement,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "lineage-replay-parity",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::ReplayParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "lineage-replay-preserves-classification",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::ReplayParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "preview-to-authoritative-identity-comparison",
        perturbation_class:
            IdentityEvolutionCertificationPerturbationClass::CorrespondenceComparison,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "identity-evolution-width-drift-explicitness",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::Performance,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "lineage-complexity-contract-parity",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::ComplexityContract,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "correspondence-complexity-contract-parity",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::ComplexityContract,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    IdentityEvolutionCanonicalRowSpec {
        row_name: "complexity-status-honesty",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::ComplexityContract,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
];

pub const IDENTITY_EVOLUTION_REJECTION_ROW_SPECS: &[IdentityEvolutionRejectionRowSpec] = &[
    IdentityEvolutionRejectionRowSpec {
        row_name: "unsupported-lineage-traversal-family",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::LineageTraversal,
        failure_class: IdentityEvolutionCertificationFailureClass::AdmissionDenied,
    },
    IdentityEvolutionRejectionRowSpec {
        row_name: "unsupported-correspondence-family",
        perturbation_class:
            IdentityEvolutionCertificationPerturbationClass::CorrespondenceComparison,
        failure_class: IdentityEvolutionCertificationFailureClass::AdmissionDenied,
    },
    IdentityEvolutionRejectionRowSpec {
        row_name: "advisory-as-authoritative-forbidden",
        perturbation_class:
            IdentityEvolutionCertificationPerturbationClass::CorrespondenceComparison,
        failure_class: IdentityEvolutionCertificationFailureClass::ExecutionDenied,
    },
    IdentityEvolutionRejectionRowSpec {
        row_name: "lineage-to-correspondence-fallback-forbidden",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::FallbackBoundary,
        failure_class: IdentityEvolutionCertificationFailureClass::ExecutionDenied,
    },
    IdentityEvolutionRejectionRowSpec {
        row_name: "branch-crossing-lineage-forbidden",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::BranchLocality,
        failure_class: IdentityEvolutionCertificationFailureClass::ExecutionDenied,
    },
    IdentityEvolutionRejectionRowSpec {
        row_name: "broad-lineage-scan-forbidden",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::Performance,
        failure_class: IdentityEvolutionCertificationFailureClass::ExecutionDenied,
    },
    IdentityEvolutionRejectionRowSpec {
        row_name: "complexity-contract-violation-denied",
        perturbation_class: IdentityEvolutionCertificationPerturbationClass::Performance,
        failure_class: IdentityEvolutionCertificationFailureClass::ExecutionDenied,
    },
];
