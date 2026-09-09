use worth_ui::facade::measurement_exchange::{
    UiMeasurementEvidenceFamily, UiViewportExtentObservation, UiViewportExtentRequest,
};
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::{
    UiHeadlessNodePaintMechanic, UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect,
    WorthUiHeadlessRecorder,
};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationStream,
};
use worth_ui_runtime::facade::entry::UiMountedAllocationMeasurementRequest;
use worth_ui_runtime::facade::host::{
    UiHostMeasurementAssumptionProfile, UiHostMeasurementNeed,
    UiHostMeasurementNormalizationContext,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRequest,
    UiMountedFrameReuse, UiMountedRgba8, UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedAllocationCertificationExt, WorthUiMountedAllocationInspectionCertificationExt,
    WorthUiMountedFrameExecutionCertificationExt, WorthUiMountedIdentityCertificationExt,
    WorthUiMountedPublicationCertificationExt,
};

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::known_empty_surface_world::profile;

struct ExpectedPulseTranscript<'a> {
    frame: worth_ui_runtime::facade::mounted::UiMountedFrameIdentity,
    surface: worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity,
    binding: worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    mounted_instances: &'a [worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity],
}

#[test]
fn in_process_real_filesystem_pulse_completes_expected_headless_rectangle() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let (mut session, source_revision, surface, binding, mounted_instances) =
        launch_and_mount_pulse(recorder.clone());
    let allocation_nodes = establish_allocation(&mut session);
    for allocation_node in &allocation_nodes {
        let live_allocation = session
            .inspect_mounted_allocation_projection(*allocation_node)
            .expect("live allocation geometry is valid");
        assert!(
            matches!(
                live_allocation,
                Some(
                    worth_ui_runtime::facade::mounted::UiMountedAllocationProjection::Known { .. }
                )
            ),
            "live allocation must be known for {allocation_node:?}, got {live_allocation:?}"
        );
    }

    let request = UiMountedFrameRequest::all_bound_surfaces();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let prepared = session
        .execute_framework_turn(|_| {})
        .expect("no presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("file-authored pulse admits ordinary execution"))
        .prepare_mounted_frame(request.clone())
        .unwrap_or_else(|denial| {
            panic!("runtime mounting completes both allocation nodes: {denial:?}")
        });
    let publication = match session.present_prepared_mounted_frame(
        prepared,
        UiPresentationDeadline::at_tick(10),
        0,
    ) {
        UiMountedFrameOutcome::Published(publication) => publication,
        _ => panic!("complete headless pulse must publish"),
    };
    assert_real_publication_projects_visible_lifecycle(&source_revision, &publication);
    assert_exact_reuse(&mut session, &request, publication.frame());
    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 1);
    assert_pulse_transcript(
        &transcripts[0],
        ExpectedPulseTranscript {
            frame: publication.frame(),
            surface,
            binding,
            mounted_instances: &mounted_instances,
        },
    );
}

fn assert_real_publication_projects_visible_lifecycle(
    source: &worth_ui::facade::source::WorthUiSourcePackageRevision,
    publication: &worth_ui_runtime::facade::mounted::UiMountedFramePublicationReceipt,
) {
    let (mut observations, started) = PlatformPulseLifecycleObservationStream::start();
    assert_eq!(started.sequence().value(), 1);
    let projected = observations
        .project_first_frame(source, publication)
        .expect("a real mounted-publication receipt projects the visible lifecycle");
    assert_eq!(projected.sequence().value(), 2);
    let PlatformPulseLifecycleObservation::FirstFramePublished(observed) = projected.outcome()
    else {
        panic!("real publication must project FirstFramePublished");
    };
    assert_eq!(
        observed.source().final_package_digest(),
        source.final_package_digest()
    );
    assert_eq!(
        observed.frame().diagnostic_value(),
        publication.frame().diagnostic_value()
    );
    assert_eq!(
        observed.generation().semantic_package_fingerprint(),
        publication
            .generation()
            .semantic_package_identity()
            .narrowing_fingerprint()
    );
}

fn assert_exact_reuse(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: &UiMountedFrameRequest,
    frame: worth_ui_runtime::facade::mounted::UiMountedFrameIdentity,
) {
    let exact_reuse = session
        .execute_framework_turn(|_| {})
        .expect("no presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("unchanged pulse remains executable"))
        .classify_mounted_frame_reuse(request);
    assert!(
        matches!(
            exact_reuse,
            UiMountedFrameReuse::Exact(ref witness) if witness.frame() == frame
        ),
        "unchanged complete paint requires the exact cloned request witness"
    );
}

fn assert_pulse_transcript(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    expected: ExpectedPulseTranscript<'_>,
) {
    assert_eq!(transcript.frame(), expected.frame);
    assert_eq!(transcript.binding(), expected.binding);
    assert_eq!(transcript.filled_rects().len(), 2);
    let mut rects = transcript.filled_rects().to_vec();
    rects.sort_by_key(|rect| rect.bounds().x() as i32);
    let background = rects[0];
    let target = rects[1];
    for rect in rects {
        assert_eq!(rect.frame(), expected.frame);
        assert_eq!(rect.surface(), expected.surface);
        assert_eq!(rect.binding(), expected.binding);
        assert!(expected
            .mounted_instances
            .contains(&rect.mounted_instance()));
        assert_eq!(rect.clip_bounds(), rect.bounds());
    }
    assert_ne!(background.mounted_instance(), target.mounted_instance());
    assert_ne!(background.node_receipt(), target.node_receipt());
    assert_eq!(background.color(), UiMountedRgba8::new(47, 129, 247, 255));
    assert_eq!(target.color(), UiMountedRgba8::new(242, 204, 96, 255));
    assert_eq!(background.layer_semantic_order(), 0);
    assert_eq!(target.layer_semantic_order(), 1);
    assert_eq!(box_values(background.bounds()), [0.0, 0.0, 160.0, 96.0]);
    assert_eq!(box_values(target.bounds()), [48.0, 24.0, 64.0, 48.0]);
    assert_eq!(
        transcript
            .nodes()
            .iter()
            .filter(|node| matches!(node.paint(), UiHeadlessNodePaintMechanic::FilledRect(_)))
            .count(),
        2
    );
    assert_eq!(
        transcript.unperformed_effects(),
        &[UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: 2,
            portal_overlay_count: 0,
            semantic_text_count: 0,
            preview_node_count: 0,
        }]
    );
}

pub(super) fn launch_and_mount_pulse(
    recorder: WorthUiHeadlessRecorder,
) -> (
    worth_ui::facade::app::WorthUiActiveApplicationSession,
    worth_ui::facade::source::WorthUiSourcePackageRevision,
    worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity,
    worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    Vec<worth_ui_runtime::facade::mounted::UiMountedInstanceIdentity>,
) {
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-2-platform-pulse");
    let workspace = FilesystemContractWorkspace::new("phase-2-platform-pulse");
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::platform_pulse_source_text(),
    );
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("production filesystem provider reads the pulse source");
    let source_revision = snapshot.source_revision().clone();
    let capabilities = scenario.platform_pulse_capability_application(recorder.clone());
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        snapshot,
        capabilities.capabilities(),
    );
    workspace.close();
    let mut session = scenario
        .prepare_platform_pulse_application_with_host(submission, recorder)
        .launch()
        .expect("query-free file-authored pulse launches");
    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    let nodes = session.graph().node_identities().collect::<Vec<_>>();
    let mounted_instances = nodes
        .into_iter()
        .map(|node| {
            let handle = session.mounted_graph_node(node).unwrap();
            session.mount_instance(handle, surface).unwrap()
        })
        .collect();
    (
        session,
        source_revision,
        surface,
        binding,
        mounted_instances,
    )
}

pub(super) fn establish_allocation(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> Vec<worth_ui::facade::graph::UiGraphNodeIdentity> {
    let capability = session.host_measurement_capability();
    let assumptions = UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    let input = UiMountedAllocationMeasurementRequest::new(
        UiMeasurementEvidenceFamily::ViewportExtent,
        UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
        UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    let receipt = session
        .establish_mounted_allocation_catalog(1, [input])
        .expect("real host viewport measurement establishes committed allocation");
    assert_eq!(receipt.committed().receipts().len(), 2);
    receipt
        .committed()
        .receipts()
        .iter()
        .map(|receipt| receipt.identity().graph_node_identity())
        .collect()
}

fn box_values(bounds: worth_ui_runtime::facade::mounted::UiMountedCanonicalBox) -> [f32; 4] {
    [bounds.x(), bounds.y(), bounds.width(), bounds.height()]
}
