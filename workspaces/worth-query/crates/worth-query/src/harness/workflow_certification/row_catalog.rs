use crate::harness::certification::HostileExpectation;

use super::{WorkflowFailureClass, WorkflowPerturbationClass};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowCanonicalRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: WorkflowPerturbationClass,
    pub hostile_expectation: HostileExpectation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowRejectionRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: WorkflowPerturbationClass,
    pub failure_class: WorkflowFailureClass,
}

pub const WORKFLOW_CANONICAL_ROW_SPECS: &[WorkflowCanonicalRowSpec] = &[
    WorkflowCanonicalRowSpec {
        row_name: "workflow-declaration-family-explicitness",
        perturbation_class: WorkflowPerturbationClass::DeclarationFamily,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-basis-family-explicitness",
        perturbation_class: WorkflowPerturbationClass::BasisFamily,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-authority-target-explicitness",
        perturbation_class: WorkflowPerturbationClass::AuthorityTargetFamily,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-preview-foundation-no-rediscovery",
        perturbation_class: WorkflowPerturbationClass::NoRediscovery,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-budget-class-explicitness",
        perturbation_class: WorkflowPerturbationClass::BudgetClass,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "query-authored-mutation-lowering-parity",
        perturbation_class: WorkflowPerturbationClass::MutationParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "query-authored-merge-lowering-parity",
        perturbation_class: WorkflowPerturbationClass::LoweringParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "query-triggered-writeback-lowering-parity",
        perturbation_class: WorkflowPerturbationClass::WritebackParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "conflict-inspection-explicitness",
        perturbation_class: WorkflowPerturbationClass::ConflictInspection,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "unsupported-deletion-topology-merge-class",
        perturbation_class: WorkflowPerturbationClass::DeniedMergeClass,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "post-merge-inspection-explicitness",
        perturbation_class: WorkflowPerturbationClass::PostMergeInspection,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-freshness-explicitness",
        perturbation_class: WorkflowPerturbationClass::Freshness,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-prediction-width-explicitness",
        perturbation_class: WorkflowPerturbationClass::PredictionWidth,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-realized-width-explicitness",
        perturbation_class: WorkflowPerturbationClass::RealizedWidth,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    WorkflowCanonicalRowSpec {
        row_name: "workflow-rediscovery-zero-parity",
        perturbation_class: WorkflowPerturbationClass::RediscoveryZero,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
];

pub const WORKFLOW_REJECTION_ROW_SPECS: &[WorkflowRejectionRowSpec] = &[
    WorkflowRejectionRowSpec {
        row_name: "unsupported-workflow-family",
        perturbation_class: WorkflowPerturbationClass::UnsupportedWorkflowFamily,
        failure_class: WorkflowFailureClass::UnsupportedWorkflowFamily,
    },
    WorkflowRejectionRowSpec {
        row_name: "invalid-basis-pairing",
        perturbation_class: WorkflowPerturbationClass::InvalidBasisPairing,
        failure_class: WorkflowFailureClass::InvalidBasisPairing,
    },
    WorkflowRejectionRowSpec {
        row_name: "preview-read-only-authority-request-forbidden",
        perturbation_class: WorkflowPerturbationClass::PreviewReadOnlyAuthority,
        failure_class: WorkflowFailureClass::PreviewReadOnlyAuthorityRequestForbidden,
    },
    WorkflowRejectionRowSpec {
        row_name: "unsupported-authority-target",
        perturbation_class: WorkflowPerturbationClass::UnsupportedAuthorityTarget,
        failure_class: WorkflowFailureClass::UnsupportedAuthorityTargetFamily,
    },
    WorkflowRejectionRowSpec {
        row_name: "forbidden-workflow-broadening",
        perturbation_class: WorkflowPerturbationClass::ForbiddenBroadening,
        failure_class: WorkflowFailureClass::ForbiddenWorkflowBroadening,
    },
    WorkflowRejectionRowSpec {
        row_name: "unsupported-merge-family",
        perturbation_class: WorkflowPerturbationClass::UnsupportedWorkflowFamily,
        failure_class: WorkflowFailureClass::UnsupportedWorkflowFamily,
    },
    WorkflowRejectionRowSpec {
        row_name: "unsupported-writeback-family",
        perturbation_class: WorkflowPerturbationClass::UnsupportedWorkflowFamily,
        failure_class: WorkflowFailureClass::UnsupportedWorkflowFamily,
    },
    WorkflowRejectionRowSpec {
        row_name: "explicit-rebind-required",
        perturbation_class: WorkflowPerturbationClass::ExplicitRebindRequired,
        failure_class: WorkflowFailureClass::ExplicitRebindRequired,
    },
    WorkflowRejectionRowSpec {
        row_name: "stale-workflow-denied",
        perturbation_class: WorkflowPerturbationClass::Freshness,
        failure_class: WorkflowFailureClass::StaleWorkflowDenied,
    },
    WorkflowRejectionRowSpec {
        row_name: "post-merge-non-authoritative-outcome-forbidden",
        perturbation_class: WorkflowPerturbationClass::PostMergeOutcomeForbidden,
        failure_class: WorkflowFailureClass::PostMergeOutcomeForbidden,
    },
];
