//! D14 derived-cache discard across graceful close and real process kill.
mod host;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::{BranchId, CommitId},
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
};
use worth_store_application::{ApplicationHomeCloseOutcome, ApplicationHomeCheckpointDenial};
use worth_store_testkit_application_oracle::{home_opening, world_home};

const LOCATION: &str = "WORTH_DERIVED_DISCARD_HOME";
const MARKERS: &str = "WORTH_DERIVED_DISCARD_MARKERS";
const MODE: &str = "WORTH_DERIVED_DISCARD_MODE";
const CHILD: &str = "process::application::host_oracle::graceful::signal::derived_discard::discard_derived_signal_child";

#[derive(serde::Serialize, serde::Deserialize)]
struct ChildSummary {
    discard_generation: u64,
    commit_id: u64,
    index_id: u64,
    index_entry_count: usize,
    record_bytes: Vec<u8>,
}

fn entity_name_index() -> DerivedIndexDefinition {
    DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "discard-derived-name-index".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                crate::batches::aspect_key("name"),
                crate::batches::field_key("name"),
            ),
        },
        branch_scoped: true,
    }
}

#[test]
fn discard_derived_graceful_close_recomputes_relational_index_and_derived_signal_on_demand() {
    run_signal_and_index_court(false);
}

#[test]
fn discard_derived_process_kill_recomputes_relational_index_and_derived_signal_on_demand() {
    run_signal_and_index_court(true);
}

fn run_signal_and_index_court(kill: bool) {
    for build in [super::FIRST, super::SECOND] {
        let root = tempfile::tempdir().unwrap();
        let location = root.path().join("home");
        let markers = root.path().join("markers");
        std::fs::create_dir(&markers).unwrap();
        let summary = run_child(&location, &markers, kill);
        verify_reopened(&location, build, &summary);
    }
}

fn run_child(location: &Path, markers: &Path, kill: bool) -> ChildSummary {
    let summary_path = markers.join("summary.json");
    if kill {
        assert_eq!(
            crate::killed_child::run_and_kill(
                CHILD,
                &[
                    (LOCATION, location),
                    (MARKERS, markers),
                    (MODE, Path::new("kill")),
                ],
                &markers.join("ready"),
                &markers.join("finished"),
            ),
            crate::killed_child::ChildEnd::KilledAtPark
        );
    } else {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(LOCATION, location)
            .env(MARKERS, markers)
            .env(MODE, "close")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "graceful discard child failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&std::fs::read(&summary_path).unwrap()).unwrap()
}

fn verify_reopened(location: &Path, build: &str, summary: &ChildSummary) {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opening = home_opening::request_with_build(
        location,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        worth_foundational::facade::ApplicationBuildIdentity::parse(build).unwrap(),
    );
    let fixture = world_home::open_with_derived_signal(opening, &execution).unwrap();
    let inspection = fixture
        .home
        .inspect_checkpoint(Default::default(), &execution)
        .unwrap();
    assert!(
        inspection.journals.iter().all(|journal| journal.covered == 0),
        "reopen after discard must read no checkpoint bytes: {inspection:?}"
    );
    assert!(inspection.journals[0].head > 0, "native journal has authored commits");
    assert!(inspection.journals[2].head > 0, "world journal has authored occurrences");
    assert!(
        inspection.native_canonical_body_reads > 0,
        "reopen after discard must replay authoritative canonical commit bodies"
    );

    let runtime = fixture.home.relational().runtime();
    let expected_def = DerivedIndexDefinition {
        index_id: DerivedIndexId(summary.index_id),
        ..entity_name_index()
    };
    assert_eq!(
        runtime.index_access().matching_definition(&expected_def),
        Some(expected_def)
    );
    let branch = BranchId("main".into());
    let generation = runtime
        .index_access()
        .read_latest_generation(&execution, DerivedIndexId(summary.index_id), &branch)
        .unwrap()
        .expect("replaying authoritative commit regenerates the registered derived index");
    assert_eq!(generation.source_commit_id, CommitId(summary.commit_id));
    assert!(summary.index_entry_count > 0);
    assert_eq!(generation.entries.len(), summary.index_entry_count);
    let rebuilt = runtime.index_authority().build_for_commit(
        &execution,
        DerivedIndexBuildRequest {
            source_commit_id: CommitId(summary.commit_id),
            branch_id: branch.clone(),
            index_ids: vec![DerivedIndexId(summary.index_id)],
        },
    );
    assert!(rebuilt.execution_denial.is_none(), "{rebuilt:?}");
    assert!(rebuilt.failed_indexes.is_empty(), "{rebuilt:?}");
    let rebuilt_gen = runtime
        .index_access()
        .read_latest_generation(&execution, DerivedIndexId(summary.index_id), &branch)
        .unwrap()
        .expect("on-demand rebuild produces a live derived index generation");
    assert_eq!(rebuilt_gen.source_commit_id, CommitId(summary.commit_id));
    assert_eq!(rebuilt_gen.entries, generation.entries);
    let actual_artifacts = crate::workload::artifacts_of(&execution, runtime);
    let expected_artifacts: Vec<Vec<u8>> = serde_json::from_slice(&summary.record_bytes).unwrap();
    assert_eq!(actual_artifacts, expected_artifacts);

    let calls = AtomicUsize::new(0);
    fixture.assert_derived_signal_authored_reference(&execution);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let expected = world_home::reference_signal_value(&execution);
    assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
    assert_eq!(calls.load(Ordering::SeqCst), 3);

    let closed = fixture
        .home
        .close(&execution)
        .unwrap_or_else(|cause| panic!("reopened Home close after discard: {:?}", cause.reason()));
    match closed {
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(checkpoint),
            ..
        } => {
            assert_eq!(checkpoint.generation(), summary.discard_generation + 1);
        }
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(cause),
            ..
        } => panic!("expected fresh checkpoint on close after reopen, got Err({cause:?})"),
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("expected closed outcome after reopen, got CloseRefused({reason:?})")
        }
    }

    // Reopening after the fresh post-discard close confirms normal checkpoint
    // selection resumes cleanly at discard_generation + 1.
    let opening_after = home_opening::request_with_build(
        location,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        worth_foundational::facade::ApplicationBuildIdentity::parse(build).unwrap(),
    );
    let reopened_after = world_home::open_with_derived_signal(opening_after, &execution).unwrap();
    let resumed_inspection = reopened_after
        .home
        .inspect_checkpoint(Default::default(), &execution)
        .unwrap();
    assert!(
        resumed_inspection.journals[0].covered > 0
            && resumed_inspection.journals[0].covered == resumed_inspection.journals[0].head,
        "fresh post-discard checkpoint must be selected on subsequent reopen: {resumed_inspection:?}"
    );
    let closed_after = reopened_after
        .home
        .close(&execution)
        .unwrap_or_else(|cause| panic!("second reopen close: {:?}", cause.reason()));
    assert!(matches!(
        closed_after,
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
}

#[test]
fn discard_derived_signal_child() {
    let Some(location) = std::env::var_os(LOCATION).map(std::path::PathBuf::from) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let mode = std::env::var(MODE).unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opening = home_opening::request(
        location,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
    );
    let fixture = world_home::open_with_derived_signal(opening, &execution).unwrap();
    let runtime = fixture.home.relational().runtime();
    let installed = runtime
        .index_authority()
        .register(entity_name_index())
        .unwrap();
    let branch = BranchId("main".into());
    let committed = crate::workload::commit(
        runtime,
        &execution,
        &branch,
        crate::batches::create(1),
    )
    .unwrap();
    let commit_id = committed.commit.commit_id;
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let built = runtime.index_authority().build_for_commit(
        &execution,
        DerivedIndexBuildRequest {
            source_commit_id: commit_id,
            branch_id: branch.clone(),
            index_ids: vec![installed.index_id],
        },
    );
    assert!(built.execution_denial.is_none(), "{built:?}");
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    let built_generation = runtime
        .index_access()
        .read_latest_generation(&execution, installed.index_id, &branch)
        .unwrap()
        .expect("built relational derived index generation");
    assert!(
        !built_generation.entries.is_empty(),
        "relational derived index must populate at least one entry"
    );
    let index_entry_count = built_generation.entries.len();
    let record_bytes =
        serde_json::to_vec(&crate::workload::artifacts_of(&execution, runtime)).unwrap();
    drop(committed);

    drop(fixture.bootstrap_root(&execution));
    fixture.fork_derived_signal_branches(&execution);
    let calls = AtomicUsize::new(0);
    let expected = world_home::reference_signal_value(&execution);
    assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
    assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
    assert_eq!(calls.load(Ordering::SeqCst), 3);

    let checkpoint = fixture.home.checkpoint(&execution).unwrap();
    assert!(checkpoint.generation() >= 1);
    let before_discard = fixture
        .home
        .inspect_checkpoint(Default::default(), &execution)
        .unwrap();
    assert!(before_discard.journals[0].covered > 0);

    let discard_generation = fixture.home.discard_derived_for_test(&execution).unwrap();
    assert_eq!(discard_generation, checkpoint.generation() + 1);
    let after_discard = fixture
        .home
        .inspect_checkpoint(Default::default(), &execution)
        .unwrap();
    assert_eq!(after_discard.journals[0].covered, 0);

    let summary = ChildSummary {
        discard_generation,
        commit_id: commit_id.0,
        index_id: installed.index_id.0,
        index_entry_count,
        record_bytes,
    };
    crate::killed_child::announce(
        &markers.join("summary.json"),
        &serde_json::to_vec(&summary).unwrap(),
    );

    if mode == "kill" {
        crate::killed_child::announce(&markers.join("ready"), b"discarded derived state");
        crate::killed_child::park();
    } else {
        let closed = fixture
            .home
            .close(&execution)
            .unwrap_or_else(|cause| panic!("graceful close after discard: {:?}", cause.reason()));
        assert!(matches!(
            closed,
            ApplicationHomeCloseOutcome::Closed {
                checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                ..
            }
        ));
    }
}
