use worth_ui_host_headless::WorthUiHeadlessHost;
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameCanonicalCore, UiMountedFrameManifest, UiMountedFramePreparationDenial,
    UiMountedFrameRequest, UiMountedLaneParticipation, UiRequiredLaneContributionStatus,
};
use worth_ui_test_support::WorthUiFrameworkTurnCertificationExt;
use worth_ui_test_support::WorthUiMountedFrameExecutionCertificationExt;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use super::mounted_application_lifecycle::known_empty_surface_world::{
    first_node, mounted_application_with_host, profile, registered_surface,
};

#[test]
fn manifest_is_exact_and_frame_identity_is_owner_minted_after_completion() {
    let mut session =
        super::mounted_application_lifecycle::known_empty_surface_world::active_session();
    let surface = registered_surface(&mut session);
    let node = first_node(&session);
    session.mount_instance(node, surface).unwrap();

    let first = prepare(&mut session, UiMountedFrameRequest::all_bound_surfaces());
    let second = prepare(&mut session, UiMountedFrameRequest::all_bound_surfaces());

    assert_eq!(first.manifest(), second.manifest());
    assert_ne!(
        first.canonical_core().frame(),
        second.canonical_core().frame()
    );
    assert!(session.inspect_mounted_identity().current_frame().is_none());
    assert_eq!(first.manifest().surfaces().len(), 1);
    assert_eq!(first.manifest().lane_contributions().len(), 5);
    assert_eq!(
        lane_status(first.manifest(), UiMountedLaneParticipation::Ordinary),
        UiRequiredLaneContributionStatus::Admitted
    );
    for lane in [
        UiMountedLaneParticipation::Virtualized,
        UiMountedLaneParticipation::CanvasSpatial,
        UiMountedLaneParticipation::Realtime,
        UiMountedLaneParticipation::Preview,
    ] {
        assert_eq!(
            lane_status(first.manifest(), lane),
            UiRequiredLaneContributionStatus::ExplicitEmpty
        );
    }
}

#[test]
fn missing_binding_denies_before_effects_and_preserves_known_predecessor() {
    let mut session =
        mounted_application_with_host("mounted-frame-missing-binding", WorthUiHeadlessHost)
            .launch()
            .unwrap();
    let bound = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            bound,
            worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    let predecessor = session.advance_mounted_identity_frame().unwrap();
    let unbound = session.create_semantic_surface().unwrap();

    let outcome = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits frame preparation"))
        .prepare_mounted_frame(UiMountedFrameRequest::exact_surfaces(vec![unbound]));
    let denial = match outcome {
        Ok(_) => panic!("an unbound semantic surface cannot become an empty contribution"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial,
        UiMountedFramePreparationDenial::MissingSurfaceBinding(unbound)
    );
    assert_eq!(
        session.inspect_mounted_identity().current_frame(),
        Some(predecessor)
    );
}

#[test]
fn manifest_order_and_integrity_are_independent_of_completion_order() {
    let mut session =
        super::mounted_application_lifecycle::known_empty_surface_world::active_session();
    registered_surface(&mut session);
    let prepared = prepare(&mut session, UiMountedFrameRequest::all_bound_surfaces());
    let expected = [
        UiMountedLaneParticipation::Ordinary,
        UiMountedLaneParticipation::Virtualized,
        UiMountedLaneParticipation::CanvasSpatial,
        UiMountedLaneParticipation::Realtime,
        UiMountedLaneParticipation::Preview,
    ];
    let cells = prepared.manifest().lane_contributions().to_vec();

    for order in permutations(cells) {
        let manifest = UiMountedFrameManifest::new(prepared.manifest().surfaces().to_vec(), order);
        assert_eq!(
            manifest
                .lane_contributions()
                .iter()
                .map(|cell| cell.lane())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(prepared
            .integrity()
            .verifies(prepared.canonical_core(), &manifest));
    }

    let core = prepared.canonical_core();
    let corrupt_ranges = UiMountedFrameCanonicalCore::new(
        core.frame(),
        core.plan_digest(),
        core.graph_world(),
        core.allocation_truth_revision(),
        core.table_range_digest() ^ 1,
    );
    assert!(!prepared
        .integrity()
        .verifies(corrupt_ranges, prepared.manifest()));
    let missing_cell = UiMountedFrameManifest::new(
        prepared.manifest().surfaces().to_vec(),
        prepared.manifest().lane_contributions()[..3].to_vec(),
    );
    assert!(!prepared
        .integrity()
        .verifies(prepared.canonical_core(), &missing_cell));
}

fn prepare(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: UiMountedFrameRequest,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits frame preparation"))
        .prepare_mounted_frame(request)
        .unwrap()
}

fn lane_status(
    manifest: &UiMountedFrameManifest,
    lane: UiMountedLaneParticipation,
) -> UiRequiredLaneContributionStatus {
    manifest
        .lane_contributions()
        .iter()
        .find(|cell| cell.lane() == lane)
        .expect("all four lane cells are explicit")
        .status()
}

fn permutations<T: Clone>(values: Vec<T>) -> Vec<Vec<T>> {
    if values.is_empty() {
        return vec![Vec::new()];
    }
    let mut results = Vec::new();
    for index in 0..values.len() {
        let mut remaining = values.clone();
        let head = remaining.remove(index);
        for mut tail in permutations(remaining) {
            tail.insert(0, head.clone());
            results.push(tail);
        }
    }
    results
}
