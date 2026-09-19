use worth_ui::facade::app::{UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline};
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountCostReport, UiMountedFrameRetentionBudget,
    UiMountedFrameRetentionBudgetInput, UiMountedRetentionClassBudget,
    UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;

#[test]
fn unchanged_mounted_turn_is_zero_work_at_one_and_1024_graph_nodes() {
    assert_zero_unchanged_graph_turn(1);
    assert_zero_unchanged_graph_turn(1_024);
}

#[test]
fn closure_stress_unchanged_mounted_turn_is_zero_work_at_65536_graph_nodes() {
    assert_zero_unchanged_graph_turn(65_536);
}

fn assert_zero_unchanged_graph_turn(graph_width: usize) {
    let label = format!("qp08-mounted-graph-{graph_width}");
    let scenario = FilesystemApplicationLifecycleScenario::new(&label);
    let workspace = FilesystemContractWorkspace::new(&label);
    let canvas_count = graph_width - 1;
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::exact_width_canvas_graph_source_text(graph_width),
    );
    let capabilities = scenario
        .scaled_canvas_capability_application(WorthUiHeadlessRecorder::default(), canvas_count);
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("scaled filesystem graph reads");
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        snapshot,
        capabilities.capabilities(),
    );
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session = scenario
        .prepare_scaled_canvas_application_with_host_and_retention_budget(
            submission,
            recorder.clone(),
            canvas_count,
            graph_retention_budget(),
        )
        .launch()
        .expect("scaled filesystem graph launches");
    assert_eq!(session.graph().node_count(), graph_width);

    let surface = session.create_semantic_surface().expect("surface mints");
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            UiSurfaceBindingProfile::new(
                1_000,
                UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .expect("bounded surface profile"),
        )
        .expect("surface registers");
    let identity = session
        .graph()
        .node_identities()
        .next()
        .expect("graph node");
    let node = session.mounted_graph_node(identity).expect("node handle");
    session
        .mount_instance(node, surface)
        .expect("one node mounts");
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let request = UiMountedFrameRequest::all_bound_surfaces();
    let first = require_published(
        session
            .execute_mounted_frame(
                request.clone(),
                UiPresentationDeadline::at_tick(1),
                0,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("initial frame executes")),
    );
    assert_eq!(first.cost_report().initial_mounted_instances(), 1);

    let unchanged = session
        .execute_mounted_frame(request, UiPresentationDeadline::at_tick(2), 1, |_| {})
        .unwrap_or_else(|_| panic!("unchanged frame executes"));
    let UiMountedFrameOutcome::Unchanged(_) = unchanged else {
        panic!("identical frame request reuses exact publication");
    };
    assert_zero_work(unchanged.cost_report().expect("unchanged cost report"));
    assert_eq!(recorder.observed_transcripts().len(), 1);

    let _ = session.shutdown();
    workspace.close();
}

fn graph_retention_budget() -> UiMountedFrameRetentionBudget {
    const MIB: usize = 1024 * 1024;
    let defaults = UiMountedFrameRetentionBudget::default();
    UiMountedFrameRetentionBudget::new(UiMountedFrameRetentionBudgetInput {
        current: UiMountedRetentionClassBudget::new(1, 128 * MIB),
        in_flight: UiMountedRetentionClassBudget::new(1, 128 * MIB),
        observation_basis: defaults.observation_basis(),
        predecessor_inspection: defaults.predecessor_inspection(),
        diagnostic: defaults.diagnostic(),
        visual_snapshot: defaults.visual_snapshot(),
        visual_overlay: defaults.visual_overlay(),
        expired_identity_limit: defaults.expired_identity_limit(),
    })
}

fn require_published(
    outcome: UiMountedFrameOutcome,
) -> worth_ui::facade::app::UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Published(receipt) => receipt,
        UiMountedFrameOutcome::Unchanged(_) => panic!("initial frame was unchanged"),
        UiMountedFrameOutcome::Reconciled(_) => panic!("initial frame reconciled"),
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("initial frame was rejected before effects")
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("initial frame remained in flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("initial frame presentation was indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!(
                "initial frame retention was denied: {:?}",
                rejection.denial()
            )
        }
        UiMountedFrameOutcome::AdmissionDenied(_) => panic!("initial frame admission was denied"),
        UiMountedFrameOutcome::CompletionDenied(_) => panic!("initial frame completion was denied"),
        UiMountedFrameOutcome::Superseded(_) => panic!("initial frame was superseded"),
    }
}

fn assert_zero_work(cost: UiMountCostReport) {
    assert_eq!(cost.initial_mounted_instances(), 0);
    assert_eq!(cost.changed_mounted_instances(), 0);
    assert_eq!(cost.index_entries_touched(), 0);
    assert_eq!(cost.replaced_batch_rows(), 0);
    assert_eq!(cost.replaced_batch_bytes(), 0);
    assert_eq!(cost.surface_instance_pairs(), 0);
    assert_eq!(cost.changed_binding_generations(), 0);
    assert_eq!(cost.adapter().presented_surfaces(), 0);
}
