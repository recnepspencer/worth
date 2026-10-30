//! A record append killed during root publication leaves root-protocol
//! candidates in record staging. Recovery removes them after its own
//! publication, so the recovered store serves instead of requiring inspection,
//! and a repeated recovery finds nothing left to remove.

#[allow(unused_imports)]
use crate::production_entry::staging_residue_recovery::*;

#[test]
fn a_record_append_killed_during_root_publication_recovers_to_a_serving_store() {
    let marker_dir = tempfile::tempdir().unwrap();
    let marker = marker_dir.path().join("ready");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, "during-root")
        .env(MARKER_ENV, &marker)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
    while !marker.is_file() {
        assert!(
            child.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline,
            "the child did not park during root publication"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let root = std::path::PathBuf::from(std::fs::read_to_string(&marker).unwrap());
    let staging = root.join("staging").join("records");
    assert!(
        std::fs::read_dir(&staging).is_ok_and(|mut entries| entries.next().is_some()),
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
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER_ENV).unwrap());
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("staging-residue").unwrap();
    append(&world, 1);
    std::thread::scope(|scope| {
        let gate = world.serving().certification_pause_physical_mutation_at(
            worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication,
        );
        scope.spawn(|| append(&world, 2));
        assert!(
            gate.await_arrival(),
            "the append did not reach root publication"
        );
        let pending = marker.with_extension("pending");
        std::fs::write(&pending, world.root().to_string_lossy().as_bytes()).unwrap();
        std::fs::rename(pending, &marker).unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
