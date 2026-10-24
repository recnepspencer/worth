//! A record append killed during root publication leaves root-protocol
//! candidates in record staging. Recovery removes them after its own
//! publication, so the recovered store serves instead of requiring inspection,
//! and a repeated recovery finds nothing left to remove.

use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationSuccess,
    PhysicalMutationRequest, RecordAppendBatch, ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

use super::*;

const ROLE_ENV: &str = "WORTH_STAGING_RESIDUE_ROLE";
const MARKER_ENV: &str = "WORTH_STAGING_RESIDUE_MARKER";
const CHILD_TEST: &str = "staging_residue_recovery::staging_residue_child";

fn append(world: &PhysicalResidencyStoreWorld, key: u8) {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([key; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"staging residue record"]).unwrap(),
                world.placement(),
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("the append must prepare")
    };
    let _ = prepared.execute();
}

fn serve(root: &std::path::Path, stage: &str) -> ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage} did not recover: {outcome:?}");
    };
    let seal = handoff.into_core().into_checkpoint_custody().unwrap();
    certified_release_serving::admit_serving_with_seal(root, seal)
}

fn assert_serving_ready(serving: &ServingPhysicalRuntime) {
    assert!(serving.publication_residue().is_empty());
    assert!(serving.physical_recovery_obligations().is_empty());
    assert!(
        serving.blobs().is_ok(),
        "the recovered store requires inspection"
    );
}

#[test]
fn a_record_append_killed_during_root_publication_recovers_to_a_serving_store() {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, "during-root")
        .env(MARKER_ENV, &marker)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(240);
    while !marker.is_file() {
        assert!(
            child.try_wait().unwrap().is_none() && Instant::now() < deadline,
            "the child did not park during root publication"
        );
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = PathBuf::from(fs::read_to_string(&marker).unwrap());
    let staging = root.join("staging").join("records");
    assert!(
        fs::read_dir(&staging).is_ok_and(|mut entries| entries.next().is_some()),
        "the killed publication must leave staging candidates"
    );
    let serving = serve(&root, "first recovery");
    assert_serving_ready(&serving);
    serving.close();
    let serving = serve(&root, "second recovery");
    assert_serving_ready(&serving);
    serving.close();
}

#[test]
fn staging_residue_child() {
    if std::env::var_os(ROLE_ENV).is_none() {
        return;
    }
    let marker = PathBuf::from(std::env::var_os(MARKER_ENV).unwrap());
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("staging-residue").unwrap();
    append(&world, 1);
    thread::scope(|scope| {
        let gate = world.serving().certification_pause_physical_mutation_at(
            CertificationPhysicalMutationCheckpoint::DuringRootPublication,
        );
        scope.spawn(|| append(&world, 2));
        assert!(
            gate.await_arrival(),
            "the append did not reach root publication"
        );
        let pending = marker.with_extension("pending");
        fs::write(&pending, world.root().to_string_lossy().as_bytes()).unwrap();
        fs::rename(pending, &marker).unwrap();
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });
}
