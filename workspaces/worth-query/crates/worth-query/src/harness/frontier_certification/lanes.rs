use crate::execution::{execute_preflight_bundle, execute_serial_fallback_route};
use crate::frontier_planning::FrontierSurfaceDigest;
use crate::harness::fixtures::execution_preflights::{
    ordered_collection_preflight, ordered_collection_without_traversal_preflight,
};
use crate::planning::{
    admit_bounded_materialization_frontier_preflight, lower_execution_preflight_to_frontier_plan,
    lower_live_plan_to_frontier_plan, lower_preflight_bundle_to_serial_fallback_routes,
    lower_preflight_to_serial_fallback_route, FrontierParityBundle, FrontierPlanFamily,
    FrontierPredictionDriftOutcome, PacketMergeContract, PlannedWorkPacketFamily,
    SerialFallbackBundleEvidence, SerialFallbackEvidence, SerialFallbackReason,
};

use super::FrontierCertificationLane;

pub(super) fn serial_control_lane() -> FrontierCertificationLane {
    let preflight = ordered_collection_without_traversal_preflight();
    let frontier_plan =
        lower_execution_preflight_to_frontier_plan(&preflight).expect("serial control plan");
    assert_eq!(
        frontier_plan.family(),
        &FrontierPlanFamily::OrderedCollection
    );
    assert_eq!(
        frontier_plan.packet_set().packets()[0].family(),
        &PlannedWorkPacketFamily::OrderedCollectionRoot
    );
    assert_eq!(
        frontier_plan.report().packet_merge_contract(),
        &PacketMergeContract::OrderedCollectionResultBoundary
    );
    let repeat =
        lower_execution_preflight_to_frontier_plan(&preflight).expect("repeat ordered plan");
    assert_eq!(
        frontier_plan.packet_set().packets()[0].digest(),
        repeat.packet_set().packets()[0].digest()
    );
    assert_eq!(
        frontier_plan.report().posture_digest(),
        repeat.report().posture_digest()
    );
    let descending =
        crate::harness::fixtures::execution_preflights::descending_collection_preflight();
    let descending_plan =
        lower_execution_preflight_to_frontier_plan(&descending).expect("descending ordered plan");
    assert_ne!(
        preflight.plan().query().plan_digest(),
        descending.plan().query().plan_digest()
    );
    assert_ne!(
        frontier_plan.packet_set().packets()[0].digest(),
        descending_plan.packet_set().packets()[0].digest()
    );
    let live = crate::live::promote_preflight_bundle_to_live(&preflight).expect("live promotion");
    let live_plan = lower_live_plan_to_frontier_plan(&live).expect("live frontier plan");
    assert_eq!(
        live_plan.source_plan_digest(),
        live.descriptor().plan_digest()
    );
    assert_eq!(live_plan.query_digest(), live.descriptor().query_digest());
    assert_eq!(
        live_plan.family(),
        &FrontierPlanFamily::LiveOrderedCollection
    );
    assert_eq!(
        live_plan.packet_set().packets()[0].family(),
        &PlannedWorkPacketFamily::LiveOrderedCollectionRoot
    );
    assert_eq!(
        live_plan.bundle_basis_digest().as_str(),
        live.progress_basis()
            .current_basis()
            .proof()
            .digest()
            .as_str()
    );
    let execution = execute_preflight_bundle(&preflight).expect("serial control execution");

    FrontierCertificationLane {
        parity_bundle: FrontierParityBundle::from_serial_control(
            &frontier_plan,
            &preflight,
            &execution,
        ),
    }
}

pub(super) fn serial_fallback_lane() -> FrontierCertificationLane {
    let preflight = ordered_collection_preflight();
    let admitted = admit_bounded_materialization_frontier_preflight(preflight.clone())
        .expect("bounded materialization admitted");
    let evidence = SerialFallbackEvidence::from_surface(
        preflight.basis().proof().digest().as_str(),
        FrontierSurfaceDigest::from_label("frontier-certification-serial-fallback"),
        SerialFallbackReason::PredictionDriftRequiresSerialRoute,
        FrontierPredictionDriftOutcome::SerialFallbackRequired,
    );
    let route =
        lower_preflight_to_serial_fallback_route(&admitted, &evidence).expect("serial route");
    assert_eq!(
        route.report().drift_outcome(),
        &FrontierPredictionDriftOutcome::SerialFallbackRequired
    );
    assert_eq!(
        route.reason(),
        &SerialFallbackReason::PredictionDriftRequiresSerialRoute
    );
    assert_eq!(route.report().serial_fallback_reason(), evidence.reason());
    let execution =
        execute_serial_fallback_route(&route).expect("serial fallback execution should succeed");
    FrontierCertificationLane {
        parity_bundle: FrontierParityBundle::from_serial_fallback(&route, &execution),
    }
}

pub(super) fn serial_fallback_bundle_lane() -> FrontierCertificationLane {
    let first = admit_bounded_materialization_frontier_preflight(ordered_collection_preflight())
        .expect("first bounded preflight admitted");
    let second = admit_bounded_materialization_frontier_preflight(ordered_collection_preflight())
        .expect("second bounded preflight admitted");
    let bundle_evidence = SerialFallbackBundleEvidence::from_routes(
        FrontierSurfaceDigest::from_label("frontier-certification-bundle"),
        vec![
            SerialFallbackEvidence::from_surface(
                first.as_preflight().basis().proof().digest().as_str(),
                FrontierSurfaceDigest::from_label("frontier-certification-bundle-a"),
                SerialFallbackReason::SerialExecutor,
                FrontierPredictionDriftOutcome::WithinBudget,
            ),
            SerialFallbackEvidence::from_surface(
                second.as_preflight().basis().proof().digest().as_str(),
                FrontierSurfaceDigest::from_label("frontier-certification-bundle-b"),
                SerialFallbackReason::SerialExecutor,
                FrontierPredictionDriftOutcome::WithinBudget,
            ),
        ],
    )
    .expect("serial fallback bundle evidence should carry one shared basis");
    let bundle = lower_preflight_bundle_to_serial_fallback_routes(
        &[first.clone(), second],
        &bundle_evidence,
    )
    .expect("serial fallback bundle should lower");
    assert_eq!(bundle.routes().len(), 2);
    assert_eq!(
        bundle.bundle_basis_digest(),
        first.as_preflight().basis().proof().digest().as_str()
    );
    for route in bundle.routes() {
        assert_eq!(
            route.preflight().basis().proof().digest().as_str(),
            bundle.bundle_basis_digest()
        );
        assert_eq!(
            route.query_digest(),
            first.as_preflight().plan().query().validated_query_digest()
        );
        assert_eq!(
            route.source_plan_digest(),
            first.as_preflight().plan().query().plan_digest()
        );
    }
    let route = &bundle.routes()[0];
    let execution = execute_serial_fallback_route(route).expect("bundle execution");
    FrontierCertificationLane {
        parity_bundle: FrontierParityBundle::from_serial_fallback_bundle(&bundle, 0, &execution)
            .expect("bundle parity bundle should resolve first route"),
    }
}
