use super::scenario::{
    admit_scroll, mixed_extent_sibling_scroll_visual_source, publish_predecessor,
    publish_scrolled_frame, publish_with_hit_coordinate, reduced_sibling_scroll_visual_source,
    with_surface_scroll_mosaic,
};
use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
};
use worth_ui::facade::observation_report::UiHostObservationPresentationBasis;
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_contract::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaSource, UiHostScrollDeltaTargetAffinity,
};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_runtime::facade::mounted::{UiMountedFrameRequest, UiPresentationDeadline};
use worth_ui_test_support::{
    UiScrollObservationCertificationOutcome, WorthUiMountedIdentityCertificationExt,
    WorthUiServiceStateCertificationExt,
};

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;
use crate::filesystem_mounted_world::{component_graph_nodes, establish_allocation};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_publication::stage_replacement;

#[test]
fn shared_surface_scroll_owner_survives_anchor_sibling_removal_and_replacement() {
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-315-shared-scroll");
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let capabilities =
        with_surface_scroll_mosaic(scenario.visual_identity_application_builder(recorder.clone()))
            .freeze()
            .expect("shared Scroll capabilities freeze");
    let initial_workspace = FilesystemContractWorkspace::new("phase-315-shared-scroll");
    initial_workspace.write("app/main.wui", &mixed_extent_sibling_scroll_visual_source());
    let snapshot = WorthUiFilesystemSourceProvider::new(initial_workspace.root())
        .read()
        .expect("production source provider reads shared Scroll source");
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        snapshot,
        capabilities.capabilities(),
    );
    let application =
        with_surface_scroll_mosaic(scenario.visual_identity_application_builder(recorder.clone()))
            .with_candidate_submission(submission)
            .freeze()
            .expect("shared Scroll application freezes");
    initial_workspace.close();

    let component_nodes = component_graph_nodes(&application);
    let mut session = application
        .launch()
        .expect("shared Scroll application launches");
    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .register_host_surface(
            surface,
            worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    let mut shared_targets = Vec::new();
    for (index, graph_node) in component_nodes.iter().copied().enumerate() {
        let handle = session.mounted_graph_node(graph_node).unwrap();
        let mounted = session.mount_instance(handle, surface).unwrap();
        if index == 1 || index == 2 {
            shared_targets.push(mounted);
        }
    }
    shared_targets.sort_by_key(|identity| identity.diagnostic_value());
    establish_allocation(&mut session, 3);
    publish_predecessor(&mut session);
    let (current, coordinate) =
        publish_with_hit_coordinate(&mut session, binding, shared_targets[0]);
    let presentation = UiHostObservationPresentationBasis::new(
        current.host_surface,
        current.frame,
        binding,
        current.epoch,
    );
    assert!(matches!(
        admit_scroll(
            &mut session,
            binding,
            &current,
            1,
            UiHostScrollDeltaPhase::Updated,
            UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, coordinate),
            0,
            -1_000,
        ),
        UiScrollObservationCertificationOutcome::Applied {
            source: UiHostScrollDeltaSource::PointerWheel,
            owners_visited: 1,
            ..
        }
    ));
    // A direct wheel offset commits when its frame is accepted.
    publish_scrolled_frame(&mut session, binding, current.instance);
    let before = session.inspect_scroll_runtime_for_certification();
    assert_eq!(before.owner_geometry().len(), 1);
    assert_eq!(before.owner_geometry()[0].block_offset_subpixels(), 1_000);

    session
        .unmount_instance(shared_targets[0])
        .expect("one shared-owner anchor sibling unmounts");
    publish_predecessor(&mut session);
    let after_unmount = session.inspect_scroll_runtime_for_certification();
    assert_eq!(after_unmount.owner_geometry().len(), 1);
    assert!(
        after_unmount.owner_geometry()[0].max_block_subpixels()
            < before.owner_geometry()[0].max_block_subpixels(),
        "the shared owner's aggregate bounds shrink when the large contributor retires"
    );
    assert_eq!(
        after_unmount.owner_geometry()[0].block_offset_subpixels(),
        before.owner_geometry()[0].block_offset_subpixels(),
        "anchor turnover clamps the shared offset instead of replacing it"
    );

    let replacement_workspace =
        FilesystemContractWorkspace::new("phase-315-shared-scroll-successor");
    replacement_workspace.write("app/main.wui", &reduced_sibling_scroll_visual_source());
    let (pending, catalog, boundary) = stage_replacement(&replacement_workspace, &mut session);
    let prepared = match session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap()
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(prepared) => prepared,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("shared-owner declaration removal requires replacement")
        }
    };
    assert!(matches!(
        prepared.present(UiPresentationDeadline::at_tick(2_000), 1),
        WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));
    replacement_workspace.close();

    let after_replacement = session.inspect_scroll_runtime_for_certification();
    assert_eq!(
        after_replacement.owner_geometry(),
        after_unmount.owner_geometry()
    );
    assert_eq!(after_replacement.owner_geometry().len(), 1);
    let _ = session.shutdown();
    drop(capabilities);
}
