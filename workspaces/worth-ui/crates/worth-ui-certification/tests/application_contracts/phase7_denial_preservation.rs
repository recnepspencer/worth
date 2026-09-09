use std::time::Duration;

use worth_ui::facade::app::{UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline};
use worth_ui::facade::source::{
    WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher,
    WorthUiSemanticHandoffPreparationStop, WorthUiWatchedCandidateSubmissionDenial,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::known_empty_surface_world::{first_node, profile};

const SOURCE: &str = "app/main.wui";
const SETTLEMENT_TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn real_watcher_distinguishes_dsl_and_runtime_denials_while_preserving_predecessor() {
    let workspace = FilesystemContractWorkspace::new("phase7-denial-preservation");
    let valid = FilesystemApplicationLifecycleScenario::ordinary_execution_source_text();
    workspace.write(SOURCE, &valid);
    let mut watcher = WorthUiFilesystemSourceWatcher::start(WorthUiFilesystemSourceProvider::new(
        workspace.root(),
    ))
    .expect("production watcher registers the real source tree");
    let scenario = FilesystemApplicationLifecycleScenario::new("phase7-denial-preservation");
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
        .prepare_application_with_host(submission, recorder)
        .launch()
        .expect("real file-authored predecessor launches");
    let predecessor = publish_predecessor(&mut session);
    let generation = session.generation_identity().clone();

    workspace.write_atomic(SOURCE, "component phase7.invalid {");
    let invalid = watcher
        .settle(SETTLEMENT_TIMEOUT)
        .expect("invalid stable bytes remain observable source truth")
        .attempt_candidate_for_certification(session.capabilities())
        .expect_err("malformed source must stop in the DSL");
    assert!(matches!(
        invalid,
        WorthUiWatchedCandidateSubmissionDenial::DslCompilation(_)
    ));
    assert_predecessor(&session, &generation, &predecessor);

    workspace.write_atomic(
        SOURCE,
        "component workspace.component.phase7_unregistered_runtime_capability {}",
    );
    let unsupported = watcher
        .settle(SETTLEMENT_TIMEOUT)
        .expect("valid unsupported bytes settle through the OS watcher")
        .attempt_candidate_for_certification(session.capabilities())
        .expect_err("valid syntax must stop at runtime capability resolution");
    assert!(matches!(
        unsupported,
        WorthUiWatchedCandidateSubmissionDenial::RuntimePreparation(denial)
            if denial.stop() == WorthUiSemanticHandoffPreparationStop::CapabilityResolution
    ));
    assert_predecessor(&session, &generation, &predecessor);

    workspace.write_atomic(SOURCE, &valid);
    watcher
        .settle(SETTLEMENT_TIMEOUT)
        .expect("restored source settles")
        .attempt_candidate_for_certification(session.capabilities())
        .expect("restored registered source crosses DSL and runtime preparation");
    assert_predecessor(&session, &generation, &predecessor);
    let _ = session.shutdown();
    let shutdown = watcher.shutdown().expect("production watcher unregisters");
    assert!(shutdown.observed_notification_count() >= 3);
    workspace.close();
}

fn publish_predecessor(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui::facade::app::UiMountedFramePublicationReceipt {
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap();
    session
        .mount_instance(first_node(session), surface)
        .unwrap();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    match session
        .execute_mounted_frame(
            UiMountedFrameRequest::all_bound_surfaces(),
            UiPresentationDeadline::at_tick(10),
            0,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("ordinary mounted predecessor executes"))
    {
        UiMountedFrameOutcome::Published(publication) => publication,
        _ => panic!("headless predecessor must publish"),
    }
}

fn assert_predecessor(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    generation: &worth_ui::facade::app::WorthUiPreparedApplicationGenerationIdentity,
    publication: &worth_ui::facade::app::UiMountedFramePublicationReceipt,
) {
    assert_eq!(session.generation_identity(), generation);
    assert_eq!(session.current_mounted_publication(), Some(publication));
}
