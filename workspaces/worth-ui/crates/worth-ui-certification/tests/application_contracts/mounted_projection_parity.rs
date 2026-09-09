use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::mounted::{
    UiMountedAllocationProjection, UiMountedFrameOutcome, UiMountedFrameRequest,
    UiMountedOmissionReason, UiMountedParticipationStatus, UiPresentationDeadline,
};
use worth_ui_test_support::WorthUiMountedFrameExecutionCertificationExt;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiFrameworkTurnCertificationExt,
};

use super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::mounted_application_lifecycle::known_empty_surface_world::{
    first_node, registered_surface,
};

#[derive(Debug, Eq, PartialEq)]
struct AuthoredMountedOracle {
    paint: UiMountedParticipationStatus,
    input: UiMountedParticipationStatus,
    focus: UiMountedParticipationStatus,
    hit_test: UiMountedParticipationStatus,
    diagnostic: UiMountedParticipationStatus,
    allocation_omission: UiMountedOmissionReason,
}

struct ProjectedMountedOracle {
    authored: AuthoredMountedOracle,
    cost: worth_ui_runtime::facade::mounted::UiMountCostReport,
    publication_transition_is_coherent: bool,
    mounted_identity_is_continuous: bool,
    transcript_count: usize,
    second_outcome: &'static str,
}

#[derive(Clone, Copy)]
enum AuthoringLane {
    File,
    Rust,
}

#[derive(Clone, Copy)]
enum QueryPosture {
    Free,
    Backed,
}

#[test]
fn file_and_rust_query_free_and_backed_worlds_match_one_mounted_contract() {
    let file_free = project_world(AuthoringLane::File, QueryPosture::Free);
    let rust_free = project_world(AuthoringLane::Rust, QueryPosture::Free);
    let file_backed = project_world(AuthoringLane::File, QueryPosture::Backed);
    let rust_backed = project_world(AuthoringLane::Rust, QueryPosture::Backed);
    let expected = AuthoredMountedOracle {
        paint: UiMountedParticipationStatus::Deferred,
        input: UiMountedParticipationStatus::Deferred,
        focus: UiMountedParticipationStatus::Deferred,
        hit_test: UiMountedParticipationStatus::Deferred,
        diagnostic: UiMountedParticipationStatus::Withheld,
        allocation_omission: UiMountedOmissionReason::NoCommittedAllocation,
    };

    for observed in [&file_free, &rust_free, &file_backed, &rust_backed] {
        assert_eq!(observed.authored, expected);
        assert!(
            observed.publication_transition_is_coherent,
            "{} transition drifted with {} transcripts",
            observed.second_outcome, observed.transcript_count,
        );
        assert!(observed.mounted_identity_is_continuous);
    }
    assert_ui_owned_cost_parity(file_free.cost, rust_free.cost);
    assert_ui_owned_cost_parity(file_backed.cost, rust_backed.cost);
    assert_ui_owned_cost_parity(file_free.cost, file_backed.cost);
    assert_eq!(
        file_free.cost.replaced_batch_rows(),
        rust_free.cost.replaced_batch_rows()
    );
    assert_eq!(
        file_backed.cost.replaced_batch_rows(),
        rust_backed.cost.replaced_batch_rows()
    );
}

fn assert_ui_owned_cost_parity(
    left: worth_ui_runtime::facade::mounted::UiMountCostReport,
    right: worth_ui_runtime::facade::mounted::UiMountCostReport,
) {
    assert_eq!(
        left.initial_mounted_instances(),
        right.initial_mounted_instances()
    );
    assert_eq!(
        left.changed_mounted_instances(),
        right.changed_mounted_instances()
    );
    assert_eq!(left.index_entries_touched(), right.index_entries_touched());
    assert_eq!(
        left.surface_instance_pairs(),
        right.surface_instance_pairs()
    );
    assert_eq!(
        left.changed_binding_generations(),
        right.changed_binding_generations()
    );
    assert_eq!(left.named().considered(), right.named().considered());
    assert_eq!(left.named().minted(), right.named().minted());
}

fn project_world(authoring: AuthoringLane, query: QueryPosture) -> ProjectedMountedOracle {
    let label = match (authoring, query) {
        (AuthoringLane::File, QueryPosture::Free) => "mounted-parity-file-free",
        (AuthoringLane::Rust, QueryPosture::Free) => "mounted-parity-rust-free",
        (AuthoringLane::File, QueryPosture::Backed) => "mounted-parity-file-query",
        (AuthoringLane::Rust, QueryPosture::Backed) => "mounted-parity-rust-query",
    };
    let mut scenario = FilesystemApplicationLifecycleScenario::new(label);
    let capabilities = scenario.capability_application();
    let submission = match authoring {
        AuthoringLane::File => file_submission(label, query, capabilities.capabilities()),
        AuthoringLane::Rust => match query {
            QueryPosture::Free => FilesystemApplicationLifecycleScenario::current_rust_submission(
                capabilities.capabilities(),
            ),
            QueryPosture::Backed => {
                FilesystemApplicationLifecycleScenario::current_query_rust_submission(
                    capabilities.capabilities(),
                )
            }
        },
    };
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session = scenario
        .prepare_application_with_host(submission, recorder.clone())
        .launch()
        .unwrap();
    if matches!(query, QueryPosture::Backed) {
        admit_query_projection(&mut scenario, &mut session);
    }
    let oracle = project_first_node(&mut session, &recorder);
    let _ = session.shutdown();
    oracle
}

fn file_submission(
    label: &str,
    query: QueryPosture,
    capabilities: &worth_ui::facade::diagnostics::CapabilitySnapshot,
) -> worth_ui::facade::source::WorthUiWatchedCandidateSubmission {
    let workspace = FilesystemContractWorkspace::new(label);
    let source = match query {
        QueryPosture::Free => FilesystemApplicationLifecycleScenario::current_source_text(),
        QueryPosture::Backed => FilesystemApplicationLifecycleScenario::current_query_source_text(),
    };
    workspace.write("app/main.wui", &source);
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .unwrap();
    workspace.close();
    FilesystemApplicationLifecycleScenario::lower_snapshot(snapshot, capabilities)
}

fn admit_query_projection(
    scenario: &mut FilesystemApplicationLifecycleScenario,
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) {
    let projection = scenario.settled_query_projection();
    let link = session.query_fact_link("inspector.measurements").unwrap();
    drop(
        session
            .execute_framework_turn(|turn| {
                turn.query_projection(|source| {
                    source.admit_settled(projection).unwrap();
                    source.submit_settled(&link).unwrap();
                });
            })
            .unwrap()
            .into_completion(),
    );
}

fn project_first_node(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    recorder: &WorthUiHeadlessRecorder,
) -> ProjectedMountedOracle {
    let surface = registered_surface(session);
    let node = first_node(session);
    let mounted_instance = session.mount_instance(node, surface).unwrap();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    let candidate = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits mounted projection"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
        .unwrap();
    let cost = candidate.cost_report();
    let view = &candidate.surfaces()[0].projection();
    let node = &view.nodes()[0];
    let participation = node.participation();
    let allocation_omission = match node.allocation() {
        UiMountedAllocationProjection::Omitted(reason) => reason,
        other => panic!("authored scenario has no committed allocation, got {other:?}"),
    };
    let authored = AuthoredMountedOracle {
        paint: participation.paint().status(),
        input: participation.input().status(),
        focus: participation.focus().status(),
        hit_test: participation.hit_test().status(),
        diagnostic: participation.diagnostic().status(),
        allocation_omission,
    };
    drop(candidate);
    let first = session
        .execute_mounted_frame(
            UiMountedFrameRequest::all_bound_surfaces(),
            UiPresentationDeadline::at_tick(10),
            0,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("first public mounted frame executes"));
    let published = match first {
        UiMountedFrameOutcome::Published(receipt) => receipt,
        _ => panic!("first public mounted frame must publish"),
    };
    let second = session
        .execute_mounted_frame(
            UiMountedFrameRequest::all_bound_surfaces(),
            UiPresentationDeadline::at_tick(11),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("unchanged public mounted frame executes"));
    let (publication_transition_is_coherent, second_outcome) = match second {
        UiMountedFrameOutcome::Unchanged(receipt) => (
            receipt.frame() == published.frame() && recorder.observed_transcripts().len() == 1,
            "unchanged",
        ),
        UiMountedFrameOutcome::Published(receipt) => (
            receipt.predecessor() == Some(published.frame())
                && recorder.observed_transcripts().len() == 1,
            "published",
        ),
        UiMountedFrameOutcome::Reconciled(_) => (false, "reconciled"),
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => (false, "rejected"),
        UiMountedFrameOutcome::InFlight(_) => (false, "in-flight"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => (false, "indeterminate"),
        UiMountedFrameOutcome::RetentionDenied(_) => (false, "retention-denied"),
        UiMountedFrameOutcome::AdmissionDenied(_) => (false, "admission-denied"),
        UiMountedFrameOutcome::CompletionDenied(_) => (false, "completion-denied"),
        UiMountedFrameOutcome::Superseded(_) => (false, "superseded"),
    };
    let identity = session.inspect_mounted_identity();
    let mounted_identity_is_continuous = identity
        .mounted_instances()
        .iter()
        .any(|receipt| receipt.identity() == mounted_instance);
    ProjectedMountedOracle {
        cost,
        authored,
        publication_transition_is_coherent,
        mounted_identity_is_continuous,
        transcript_count: recorder.observed_transcripts().len(),
        second_outcome,
    }
}
