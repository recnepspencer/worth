use std::time::Duration;

use worth_ui::facade::observation::{UiChangeClassificationOutcome, UiObservationAdmissionDenial};
use worth_ui::facade::source::{
    UiSourceRebindAttemptOutcome, WorthUiCandidateOrderingReceipt, WorthUiFilesystemSourceProvider,
    WorthUiFilesystemSourceWatcher, WorthUiSourcePackageRevision,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;

#[test]
fn settled_revision_survives_compile_admission_and_a_later_file_write() {
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-312-tt02-affinity");
    let workspace = FilesystemContractWorkspace::new("phase-312-tt02-affinity");
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::platform_pulse_source_text_with_unrelated_width(0),
    );
    let mut watcher = WorthUiFilesystemSourceWatcher::start(WorthUiFilesystemSourceProvider::new(
        workspace.root(),
    ))
    .expect("production filesystem watcher starts");
    let initial = watcher
        .take_initial_snapshot()
        .expect("watcher owns initial settled revision");
    let stale_snapshot = initial.clone();
    let recorder = WorthUiHeadlessRecorder::default();
    let capabilities =
        scenario.platform_pulse_capability_application_with_unrelated_width(recorder.clone(), 0);
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        initial,
        capabilities.capabilities(),
    );
    let mut session = scenario
        .prepare_platform_pulse_application_with_unrelated_width(submission, recorder, 0)
        .launch()
        .expect("filesystem-authored application launches");

    workspace.write_atomic(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::
            platform_pulse_green_source_text_with_unrelated_width(0),
    );
    let green = watcher
        .settle(Duration::from_secs(5))
        .expect("green edit settles exactly once");
    let green_revision = green.source_revision().clone();
    let green_ordering = green.ordering_receipt().clone();

    workspace.write_atomic("app/main.wui", "component workspace.component.broken {");
    let green_candidate = green
        .attempt_source_rebind(session.capabilities())
        .into_candidate_submission()
        .expect("held green snapshot compiles without rereading later malformed bytes");
    assert_eq!(green_candidate.source_revision(), &green_revision);
    assert_eq!(green_candidate.ordering_receipt(), &green_ordering);
    assert_exact_revision_affinity(&green_ordering, &green_revision);

    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(green_candidate).unwrap();
    let admitted = turn.seal().unwrap();
    let source = admitted.observations()[0]
        .source_observation()
        .expect("source observation remains typed");
    assert_eq!(source.revision(), &green_revision);
    assert_eq!(source.ordering_receipt(), &green_ordering);
    assert!(matches!(
        session.classify_observations(admitted),
        Ok(UiChangeClassificationOutcome::Changed(_))
    ));

    let stale_candidate = stale_snapshot
        .attempt_source_rebind(session.capabilities())
        .into_candidate_submission()
        .expect("initial snapshot remains a valid but historical candidate");
    let mut stale_turn = session.begin_observation_turn().unwrap();
    assert_eq!(
        stale_turn.admit_source(stale_candidate),
        Err(UiObservationAdmissionDenial::HistoricalOwnerOrder)
    );
    drop(stale_turn);

    let malformed = watcher
        .settle(Duration::from_secs(5))
        .expect("later malformed bytes settle as their own revision");
    let malformed_revision = malformed.source_revision().clone();
    let malformed_ordering = malformed.ordering_receipt().clone();
    let denial = match malformed.attempt_source_rebind(session.capabilities()) {
        UiSourceRebindAttemptOutcome::CompilationDenied(receipt) => receipt,
        _ => panic!("malformed revision must stop at DSL compilation"),
    };
    assert_eq!(denial.source_revision(), &malformed_revision);
    assert_eq!(denial.ordering_receipt(), &malformed_ordering);
    assert_exact_revision_affinity(&malformed_ordering, &malformed_revision);
    assert!(!denial.report().diagnostics().is_empty());

    let _ = session.shutdown();
    watcher.shutdown().expect("watcher shuts down");
    workspace.close();
}

#[test]
fn candidate_compiled_for_a_foreign_capability_basis_denies_before_admission() {
    let current = FilesystemApplicationLifecycleScenario::new("phase-312-tt02-current");
    let current_workspace = FilesystemContractWorkspace::new("phase-312-tt02-current");
    current_workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::platform_pulse_source_text_with_unrelated_width(0),
    );
    let recorder = WorthUiHeadlessRecorder::default();
    let current_capabilities =
        current.platform_pulse_capability_application_with_unrelated_width(recorder.clone(), 0);
    let current_submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        WorthUiFilesystemSourceProvider::new(current_workspace.root())
            .read()
            .expect("current filesystem revision reads"),
        current_capabilities.capabilities(),
    );
    let mut session = current
        .prepare_platform_pulse_application_with_unrelated_width(current_submission, recorder, 0)
        .launch()
        .expect("current application launches");

    let foreign = FilesystemApplicationLifecycleScenario::new("phase-312-tt02-foreign");
    let foreign_workspace = FilesystemContractWorkspace::new("phase-312-tt02-foreign");
    foreign_workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::platform_pulse_source_text_with_unrelated_width(1),
    );
    let foreign_capabilities = foreign.platform_pulse_capability_application_with_unrelated_width(
        WorthUiHeadlessRecorder::default(),
        1,
    );
    let foreign_candidate = WorthUiFilesystemSourceProvider::new(foreign_workspace.root())
        .read()
        .expect("foreign filesystem revision reads")
        .attempt_source_rebind(foreign_capabilities.capabilities())
        .into_candidate_submission()
        .expect("foreign candidate is valid for its own capability world");

    let mut turn = session.begin_observation_turn().unwrap();
    assert_eq!(
        turn.admit_source(foreign_candidate),
        Err(UiObservationAdmissionDenial::ForeignSourceBasis)
    );
    assert!(matches!(
        turn.seal(),
        Err(UiObservationAdmissionDenial::PoisonedTurn)
    ));
    let _ = session.shutdown();
    foreign_workspace.close();
    current_workspace.close();
}

fn assert_exact_revision_affinity(
    ordering: &WorthUiCandidateOrderingReceipt,
    revision: &WorthUiSourcePackageRevision,
) {
    assert_eq!(ordering.provider_id(), revision.provider_id());
    assert_eq!(
        ordering.source_revision_digest(),
        revision.final_package_digest()
    );
    assert_eq!(ordering.event_burst_digest(), revision.event_burst_digest());
    assert_eq!(ordering.sequence(), revision.sequence());
}
