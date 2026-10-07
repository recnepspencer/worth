use crate::harness::certification::HostileExpectation;

use super::{FrontierFailureClass, FrontierPerturbationClass, FrontierRouteClass};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontierCanonicalRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: FrontierPerturbationClass,
    pub hostile_expectation: HostileExpectation,
    pub route_class: FrontierRouteClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontierRejectionRowSpec {
    pub row_name: &'static str,
    pub perturbation_class: FrontierPerturbationClass,
    pub failure_class: FrontierFailureClass,
}

pub const FRONTIER_CANONICAL_ROW_SPECS: &[FrontierCanonicalRowSpec] = &[
    FrontierCanonicalRowSpec {
        row_name: "frontier-serial-control",
        perturbation_class: FrontierPerturbationClass::SerialControlParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
        route_class: FrontierRouteClass::SerialControl,
    },
    FrontierCanonicalRowSpec {
        row_name: "serial-fallback-parity",
        perturbation_class: FrontierPerturbationClass::SerialFallbackParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
        route_class: FrontierRouteClass::SerialFallback,
    },
    FrontierCanonicalRowSpec {
        row_name: "exact-basis-bundle-parity",
        perturbation_class: FrontierPerturbationClass::ExactBasisBundleParity,
        hostile_expectation: HostileExpectation::EquivalentToControl,
        route_class: FrontierRouteClass::SerialFallbackBundle,
    },
];

pub const FRONTIER_REJECTION_ROW_SPECS: &[FrontierRejectionRowSpec] = &[
    FrontierRejectionRowSpec {
        row_name: "unsupported-frontier-family",
        perturbation_class: FrontierPerturbationClass::UnsupportedFrontierFamilyRejection,
        failure_class: FrontierFailureClass::UnsupportedFrontierFamily,
    },
    FrontierRejectionRowSpec {
        row_name: "unsupported-bundle-composition",
        perturbation_class: FrontierPerturbationClass::UnsupportedBundleCompositionRejection,
        failure_class: FrontierFailureClass::UnsupportedBundleComposition,
    },
    FrontierRejectionRowSpec {
        row_name: "mixed-basis-bundle-denied",
        perturbation_class: FrontierPerturbationClass::MixedBasisBundleRejection,
        failure_class: FrontierFailureClass::MixedBasisBundleDenied,
    },
];

pub const FRONTIER_REQUIRED_CANONICAL_ROW_NAMES: &[&str] = &[
    "frontier-serial-control",
    "serial-fallback-parity",
    "exact-basis-bundle-parity",
];

pub const FRONTIER_REQUIRED_REJECTION_ROW_NAMES: &[&str] = &[
    "unsupported-frontier-family",
    "unsupported-bundle-composition",
    "mixed-basis-bundle-denied",
];
