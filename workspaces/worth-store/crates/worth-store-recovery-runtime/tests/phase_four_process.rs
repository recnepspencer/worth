#[allow(dead_code)]
mod phase_three_support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use phase_three_support::{
    admitted_recovery_with_limits, limit_declaration, recovery_request_with_limits,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    recovery_wal::WalSegmentArtifactIdentity, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::{
    PhysicalRedoDecisionKind, PhysicalRedoDecisionPrior, PhysicalRedoTargetIdentity,
    RecoveryOperationFate, RecoveryPageSource,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, RecoveryPublicationAction, WorthStoreRecovery,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    canonical_rooted_mutation_without_acknowledgment, PhysicalResidencyStoreWorld,
};

const CHILD_ROOT: &str = "WORTH_C8_PHASE4_CHILD_ROOT";
const WRITER_TEST: &str = "phase_four_writer_process";
const PLANNER_TEST: &str = "phase_four_planner_process";
const FACADE_TEST: &str = "phase_six_facade_process";

#[test]
fn ordinary_store_state_crosses_process_death_into_one_recovered_handoff() {
    let parent = tempfile::tempdir().expect("process-boundary parent");
    let marker = parent.path().join("persisted-root");
    let writer = run_child(WRITER_TEST, &marker, parent.path());
    assert_child_succeeded("writer", &writer);

    let root = PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    assert!(root.starts_with(parent.path()));
    assert!(root.is_dir());
    append_torn_terminal_wal_segment(&root);
    let planner = run_child(PLANNER_TEST, &root, parent.path());
    assert_child_succeeded("planner", &planner);
}

#[test]
fn production_facade_replaces_a_dead_writer_in_a_distinct_process() {
    let parent = tempfile::tempdir().expect("facade process-boundary parent");
    let marker = parent.path().join("persisted-root");
    let writer = run_child(WRITER_TEST, &marker, parent.path());
    assert_child_succeeded("writer", &writer);

    let root = PathBuf::from(std::fs::read_to_string(&marker).expect("writer root marker"));
    let facade = run_child(FACADE_TEST, &root, parent.path());
    assert_child_succeeded("facade", &facade);
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn phase_four_writer_process() {
    let marker = required_child_path();
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("c8-phase4-process").unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"ordinary-c8-redo");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary checkpoint admission must succeed")
    };
    let checkpoint = match handle.wait() {
        PhysicalCheckpointOutcome::Completed(checkpoint) => checkpoint,
        other => panic!("ordinary checkpoint publication must complete: {other:?}"),
    };
    assert!(checkpoint.retained_wal_tail().segment_count().get() > 0);
    canonical_rooted_mutation_without_acknowledgment(&world, [0x43; 32], b"rooted-c8-redo");
    canonical_durable_wal_attempt_without_execution(&world, [0x44; 32], b"post-checkpoint-c8-redo");
    drop(world);
    let root = retained_root.persist();
    std::fs::write(marker, root.to_string_lossy().as_bytes()).expect("persisted root marker");
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn phase_four_planner_process() {
    let root = required_child_path();
    let planned = admitted_recovery_with_limits(&root, ordinary_limits())
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap();

    let fates = planned.operation_fates().operations();
    // The parent added a poisoned terminal frame after writer process death.
    // C.8 still selects its admissible prefix, with no raw WAL owner decoder entry.
    assert_eq!(planned.discovery_counters().wal_owner_decoder_entries, 0);
    assert!(planned.discovery_counters().wal_integrity_rejections > 0);
    assert_eq!(fates.len(), 3);
    assert_eq!(fates[0].fate(), RecoveryOperationFate::AcknowledgedDurable);
    assert_eq!(
        fates[1].fate(),
        RecoveryOperationFate::DurableUnacknowledged
    );
    assert_eq!(fates[2].fate(), RecoveryOperationFate::Indeterminate);
    let decisions = planned.redo_plan().resolved_decisions().collect::<Vec<_>>();
    assert_eq!(decisions.len(), 2);
    assert_eq!(
        decisions[0].kind(),
        PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
    );
    assert_eq!(decisions[1].kind(), PhysicalRedoDecisionKind::Apply);
    assert_eq!(decisions[1].record().lsn().get(), 3);
    assert_eq!(
        decisions[1].target().identity(),
        PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 1,
            generation: 3,
        }
    );
    let PhysicalRedoDecisionPrior::Page(prior) = decisions[1].prior() else {
        panic!("an applicable redo step retains its exact selected-page proof")
    };
    let RecoveryPageSource::Materialized {
        coordinate,
        routing_identity,
    } = prior.source()
    else {
        panic!("the ordinary successor is based on the selected materialized page")
    };
    assert_eq!(
        coordinate.artifact().file_name(),
        "segment-0000000000000001-0000000000000002.pages"
    );
    assert_ne!(routing_identity, [0; 32]);

    let cost = planned.plan_cost();
    assert_eq!(cost.redo_targets(), 2);
    assert_eq!(cost.redo_bytes(), 34_258);
    assert_eq!(cost.distinct_targets(), 2);
    assert_eq!(cost.operation_bindings(), 3);
    assert_eq!(cost.observation_reads(), 7);
    assert_eq!(cost.observation_bytes(), 73_275);
    assert_eq!(cost.staging_bytes(), 3_276_800);
    assert_eq!(cost.dirty_frames(), 1);
    assert_eq!(planned.staging_layout().actions().len(), 1);
    assert_eq!(planned.staging_layout().actions()[0].steps().len(), 1);
    assert!(planned
        .staging_layout()
        .commands()
        .iter()
        .any(|command| command.byte_count() == 16_384));
    let base = planned.staging_layout().base_image();
    assert_eq!(base.actions().len(), 3);
    assert!(base.actions().iter().all(|action| action.is_projected()));
    assert_eq!(base.segment_updates().len(), 1);
    let publication = planned.publication_plan();
    assert!(!publication.candidates().is_empty());
    assert_eq!(
        publication.actions().len(),
        publication.candidates().len() * 2 + 2
    );
    assert_eq!(
        &publication.actions()[publication.actions().len() - 2..],
        [
            RecoveryPublicationAction::ReplaceRootProtocol,
            RecoveryPublicationAction::SynchronizeStoreNamespace,
        ]
    );
    assert_ne!(planned.publication_plan().plan_identity(), [0; 32]);
    let protocol = planned.publication_plan().root_protocol();
    assert_eq!(
        protocol.catalog_candidate(),
        RecordArtifactFile::CatalogCandidate {
            publication: protocol.publication(),
        }
    );

    let expected_generation = planned.publication_plan().staging_generation();
    let expected_plan = planned.publication_plan().plan_identity();
    let reopened = planned
        .stage()
        .unwrap()
        .publish()
        .unwrap()
        .reopen()
        .unwrap();
    assert_eq!(reopened.recovered_root().generation(), expected_generation);
    assert_eq!(
        reopened.publication_expectation().plan_identity(),
        expected_plan
    );
    assert_eq!(reopened.reopen_counters().selector_reads_completed, 1);
    assert_eq!(reopened.reopen_counters().root_reads_completed, 1);
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("process-death recovery must produce the recovered handoff")
    };
    assert_eq!(handoff.core().root().generation(), expected_generation);
    assert_ne!(
        handoff.core().runtime_identity(),
        handoff.core().recovery_runtime_identity()
    );
    assert_eq!(handoff.operation_fates().operations().len(), 3);
    assert_eq!(handoff.discovery_counters().wal_owner_decoder_entries, 0);
    assert_eq!(
        handoff.integrity_observation_count() as usize,
        handoff.integrity_observations().len()
    );
    assert!(handoff.integrity_observations().iter().any(|observation|
        observation.scope().artifact_family()
            == worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter));
    assert!(handoff
        .wal_integrity_observations()
        .iter()
        .any(|observation| matches!(
            observation.outcome(),
            worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
        )));
    assert!(handoff
        .wal_integrity_observations()
        .iter()
        .any(|observation| matches!(
            observation.outcome(),
            worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(
                _
            )
        )));
    assert!(handoff.core().recovery_effect_count() > 0);
}

fn append_torn_terminal_wal_segment(root: &Path) {
    let wal = root.join("families").join("wal");
    let latest = std::fs::read_dir(&wal)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            WalSegmentArtifactIdentity::parse(&name).map(|identity| (identity, entry.path()))
        })
        .max_by_key(|(identity, _)| *identity)
        .unwrap();
    let next_segment = latest.0.segment().get() + 1;
    let (path, frame) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &root.join("families"),
            next_segment,
            0,
            1,
            "terminal-torn-observation",
            b"not-owner-visible",
        );
    std::fs::write(path, &frame[..37]).unwrap();
}

#[test]
#[ignore = "launched by the process-boundary parent"]
fn phase_six_facade_process() {
    let root = required_child_path();
    let request = recovery_request_with_limits(&root, ordinary_limits());
    let PhysicalRecoveryOutcome::Recovered(handoff) = WorthStoreRecovery::recover(request) else {
        panic!("the production facade must recover the dead ordinary writer")
    };
    assert_ne!(
        handoff.core().runtime_identity(),
        handoff.core().recovery_runtime_identity()
    );
    assert_eq!(handoff.reopen_counters().selector_reads_completed, 1);
    assert_eq!(handoff.reopen_counters().root_reads_completed, 1);
    assert!(handoff.core().recovery_effect_count() > 0);
}

fn ordinary_limits() -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 2 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 4 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 32 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.observation_bytes = 32 * 1024 * 1024;
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

fn run_child(test: &str, path: &Path, temporary_root: &Path) -> Output {
    let mut command = Command::new(std::env::current_exe().expect("integration test executable"));
    command
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env(CHILD_ROOT, path)
        .env("TMP", temporary_root)
        .env("TEMP", temporary_root)
        .env("TMPDIR", temporary_root);
    command.output().expect("launch Phase 4 child process")
}

fn required_child_path() -> PathBuf {
    std::env::var_os(CHILD_ROOT)
        .map(PathBuf::from)
        .expect("Phase 4 child root")
}

fn assert_child_succeeded(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
