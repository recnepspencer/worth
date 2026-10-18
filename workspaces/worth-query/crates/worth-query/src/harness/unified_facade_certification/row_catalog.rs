use super::{UnifiedFacadeFailureClass, UnifiedFacadePerturbationClass};
use crate::harness::certification::HostileExpectation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnifiedFacadeCanonicalRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: UnifiedFacadePerturbationClass,
    pub hostile_expectation: HostileExpectation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnifiedFacadeRejectionRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: UnifiedFacadePerturbationClass,
    pub failure_class: UnifiedFacadeFailureClass,
}

pub const UNIFIED_FACADE_CANONICAL_ROW_SPECS: &[UnifiedFacadeCanonicalRowSpec] = &[
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-query-read-capability",
        perturbation_class: UnifiedFacadePerturbationClass::ApplicationCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-query-context-capability",
        perturbation_class: UnifiedFacadePerturbationClass::QueryContextCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-identity-evolution-capability",
        perturbation_class: UnifiedFacadePerturbationClass::ApplicationCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-query-context-basis-result-bundle",
        perturbation_class: UnifiedFacadePerturbationClass::QueryContextCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-query-context-diff-result-bundle",
        perturbation_class: UnifiedFacadePerturbationClass::QueryContextCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-live-capability",
        perturbation_class: UnifiedFacadePerturbationClass::ApplicationCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-preview-capability",
        perturbation_class: UnifiedFacadePerturbationClass::ApplicationCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-workflow-capability",
        perturbation_class: UnifiedFacadePerturbationClass::ApplicationCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-historical-capability",
        perturbation_class: UnifiedFacadePerturbationClass::ApplicationCapability,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "unified-config-section-explicitness",
        perturbation_class: UnifiedFacadePerturbationClass::ConfigurationSection,
        hostile_expectation: HostileExpectation::DistinctFromControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "capability-support-metadata-sync",
        perturbation_class: UnifiedFacadePerturbationClass::SupportMetadata,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "query-context-support-profile-sync",
        perturbation_class: UnifiedFacadePerturbationClass::SupportMetadata,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
    UnifiedFacadeCanonicalRowSpec {
        row_name: "identity-evolution-support-profile-sync",
        perturbation_class: UnifiedFacadePerturbationClass::SupportMetadata,
        hostile_expectation: HostileExpectation::EquivalentToControl,
    },
];

pub const UNIFIED_FACADE_REJECTION_ROW_SPECS: &[UnifiedFacadeRejectionRowSpec] = &[
    UnifiedFacadeRejectionRowSpec {
        row_name: "missing-owning-live-section",
        perturbation_class: UnifiedFacadePerturbationClass::UnsupportedComposition,
        failure_class: UnifiedFacadeFailureClass::MissingOwningSection,
    },
    UnifiedFacadeRejectionRowSpec {
        row_name: "invalid-workflow-support-posture",
        perturbation_class: UnifiedFacadePerturbationClass::UnsupportedComposition,
        failure_class: UnifiedFacadeFailureClass::InvalidComposedSupportPosture,
    },
    UnifiedFacadeRejectionRowSpec {
        row_name: "deferred-durable-artifacts",
        perturbation_class: UnifiedFacadePerturbationClass::DeferredComposition,
        failure_class: UnifiedFacadeFailureClass::DeferredCapability,
    },
    UnifiedFacadeRejectionRowSpec {
        row_name: "invalid-unified-configuration",
        perturbation_class: UnifiedFacadePerturbationClass::ConfigurationSection,
        failure_class: UnifiedFacadeFailureClass::InvalidConfiguration,
    },
    UnifiedFacadeRejectionRowSpec {
        row_name: "broad-collection-diff-denied",
        perturbation_class: UnifiedFacadePerturbationClass::QueryContextCapability,
        failure_class: UnifiedFacadeFailureClass::QueryContextBroadeningDenied,
    },
];
