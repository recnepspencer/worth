use crate::harness::certification::{
    CanonicalCertificationRow, ParityAnchor, RejectionCertificationRow,
};

use super::rejections::{
    mixed_basis_bundle_rejection, unsupported_bundle_composition_rejection,
    unsupported_frontier_family_rejection,
};
use super::row_catalog;
use super::{
    FrontierCertificationLane, FrontierCertificationRejection, FrontierPerturbationClass,
    FrontierRouteClass,
};

pub(super) fn canonical_row(
    spec: &row_catalog::FrontierCanonicalRowSpec,
    serial_control: &FrontierCertificationLane,
    serial_fallback: &FrontierCertificationLane,
    bundle_lane: &FrontierCertificationLane,
) -> CanonicalCertificationRow<FrontierPerturbationClass, FrontierCertificationLane> {
    let lane = match spec.route_class {
        FrontierRouteClass::SerialControl => serial_control.clone(),
        FrontierRouteClass::SerialFallback => serial_fallback.clone(),
        FrontierRouteClass::SerialFallbackBundle => bundle_lane.clone(),
    };
    CanonicalCertificationRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        hostile_expectation: spec.hostile_expectation,
        parity_anchor: ParityAnchor::Control,
        control_lane: lane.clone(),
        hostile_lane: lane.clone(),
        parity_lane: lane,
    }
}

pub(super) fn rejection_row(
    spec: &row_catalog::FrontierRejectionRowSpec,
    serial_control: &FrontierCertificationLane,
) -> RejectionCertificationRow<
    FrontierPerturbationClass,
    FrontierCertificationLane,
    FrontierCertificationRejection,
> {
    let hostile_lane = match spec.row_name {
        "unsupported-frontier-family" => unsupported_frontier_family_rejection(),
        "unsupported-bundle-composition" => unsupported_bundle_composition_rejection(),
        "mixed-basis-bundle-denied" => mixed_basis_bundle_rejection(),
        other => panic!("unknown frontier rejection row {other}"),
    };

    RejectionCertificationRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        control_lane: serial_control.clone(),
        hostile_lane,
        parity_lane: serial_control.clone(),
    }
}
