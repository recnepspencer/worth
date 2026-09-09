use std::fs;

use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedFrameRequest,
    UiPresentationDeadline,
};
use worth_ui::facade::source::{WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::application::WorthUiOrdinaryFrameTarget;
use worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedIdentityCertificationExt,
};

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::known_empty_surface_world::{first_node, profile};

const SOURCE: &str = "app/main.wui";
const POISONED_SOURCE: &str = "component phase7.hot_frame_poison {";

#[test]
fn poisoned_watched_source_cannot_enter_unchanged_or_changed_mounted_frames() {
    let workspace = FilesystemContractWorkspace::new("phase7-hot-frame-source-exclusion");
    workspace.write(
        SOURCE,
        &FilesystemApplicationLifecycleScenario::ordinary_execution_source_text(),
    );
    let mut watcher = WorthUiFilesystemSourceWatcher::start(WorthUiFilesystemSourceProvider::new(
        workspace.root(),
    ))
    .expect("production watcher registers the real source tree");
    let scenario = FilesystemApplicationLifecycleScenario::new("phase7-hot-frame-source-exclusion");
    let capabilities = scenario.capability_application();
    let initial = watcher
        .take_initial_snapshot()
        .expect("watcher owns the initial settled source");
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        initial,
        capabilities.capabilities(),
    );
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session = scenario
        .prepare_application_with_host(submission, recorder.clone())
        .launch()
        .expect("real file-authored application launches");
    let surface = mount_one_instance(&mut session);

    let request = UiMountedFrameRequest::all_bound_surfaces();
    let first = published(
        session
            .execute_mounted_frame(
                request.clone(),
                UiPresentationDeadline::at_tick(10),
                0,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("initial mounted frame executes")),
    );
    let active_generation = session.generation_identity().clone();
    let active_plan = session.inspect_runtime().active_plan_digest();
    assert_eq!(first.generation(), &active_generation);
    assert_eq!(recorder.observed_transcripts().len(), 1);

    workspace.write_atomic(SOURCE, POISONED_SOURCE);
    assert_eq!(
        fs::read_to_string(workspace.path(SOURCE)).expect("poisoned file remains readable"),
        POISONED_SOURCE,
        "the independent disk oracle must observe bytes that cannot pass DSL parsing"
    );

    for tick in 11..=12 {
        let outcome = session
            .execute_mounted_frame(
                request.clone(),
                UiPresentationDeadline::at_tick(tick),
                tick,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("unchanged frame cannot consult poisoned source"));
        let cost = outcome
            .cost_report()
            .expect("unchanged outcome carries its zero-work cost");
        assert_eq!(cost.initial_mounted_instances(), 0);
        assert_eq!(cost.changed_mounted_instances(), 0);
        assert_eq!(cost.index_entries_touched(), 0);
        assert_eq!(cost.surface_instance_pairs(), 0);
        assert_eq!(cost.adapter().presented_surfaces(), 0);
        assert_eq!(cost.named().reused(), 1);
        let unchanged = unchanged(outcome);
        assert_eq!(unchanged, first);
    }
    assert_eq!(
        recorder.observed_transcripts().len(),
        1,
        "exact reuse has no adapter consequence"
    );

    let execution = session
        .execute_framework_turn(|_| {})
        .expect("poisoned disk bytes cannot affect an active framework turn")
        .into_execution()
        .unwrap_or_else(|_| panic!("active generation remains executable"));
    let ordinary = execution
        .execute_ordinary_frame(WorthUiOrdinaryFrameTarget::root_shell())
        .expect("ordinary lane executes from sealed active truth");
    let cost = ordinary
        .cost_receipt()
        .expect("generation-bound steady-frame counters certify");
    assert_eq!(cost.active_plan_digest(), active_plan);
    assert_eq!(cost.counters().total_forbidden_source_or_registry_work(), 0);
    assert!(cost
        .lane_receipts()
        .iter()
        .all(|lane| lane.work_scope().is_within_request()));
    drop(ordinary);
    drop(execution);

    let node = first_node(&session);
    session
        .mount_instance(node, surface)
        .expect("one-instance semantic delta remains valid");
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let changed = published(
        session
            .execute_mounted_frame(request, UiPresentationDeadline::at_tick(30), 3, |_| {})
            .unwrap_or_else(|_| panic!("changed frame cannot consult poisoned source")),
    );
    assert_eq!(session.generation_identity(), &active_generation);
    assert_eq!(changed.generation(), &active_generation);
    assert_eq!(changed.predecessor(), Some(first.frame()));
    assert_eq!(changed.cost_report().changed_mounted_instances(), 1);
    assert_eq!(changed.cost_report().changed_binding_generations(), 0);
    assert_eq!(changed.cost_report().adapter().presented_surfaces(), 1);
    assert_eq!(session.current_mounted_publication(), Some(&changed));
    let transcripts = recorder.observed_transcripts();
    assert_eq!(transcripts.len(), 2);
    assert_eq!(transcripts[1].frame(), changed.frame());

    let _ = session.shutdown();
    watcher
        .shutdown()
        .expect("production watcher unregisters without settling poisoned bytes");
    workspace.close();
}

fn mount_one_instance(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity {
    let surface = session
        .create_semantic_surface()
        .expect("semantic surface identity mints");
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .expect("headless surface registers");
    session
        .mount_instance(first_node(session), surface)
        .expect("one active graph node mounts");
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    surface
}

fn published(outcome: UiMountedFrameOutcome) -> UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Published(publication) => publication,
        UiMountedFrameOutcome::Unchanged(_) => panic!("headless frame was unchanged"),
        UiMountedFrameOutcome::Reconciled(_) => panic!("headless frame reconciled"),
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("headless frame was rejected before effects")
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("headless frame remained in flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("headless frame presentation was indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(_) => panic!("headless frame retention denied"),
        UiMountedFrameOutcome::AdmissionDenied(_) => panic!("headless frame admission denied"),
        UiMountedFrameOutcome::CompletionDenied(_) => panic!("headless frame completion denied"),
        UiMountedFrameOutcome::Superseded(_) => panic!("headless frame was superseded"),
    }
}

fn unchanged(outcome: UiMountedFrameOutcome) -> UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Unchanged(publication) => publication,
        _ => panic!("identical request over the same active truth must reuse exactly"),
    }
}
