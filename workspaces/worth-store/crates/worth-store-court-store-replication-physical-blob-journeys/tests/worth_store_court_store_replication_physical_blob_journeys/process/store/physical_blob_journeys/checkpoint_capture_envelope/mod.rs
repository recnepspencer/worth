//! A checkpoint capture runs on the Store's standing reservation at a
//! realistic head population: it takes no new Recovery bytes, and its real
//! heap allocations fit the envelope that reservation holds. The child also reports the process heap peak; the
//! dirty-frame slices, media and signal work it includes are separately owned
//! and outside this envelope.

#[allow(unused_imports)]
use worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::*;

#[test]
fn checkpoint_capture_runs_on_its_standing_reservation_and_its_heap_fits_the_envelope() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    std::fs::create_dir(&root).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::store::physical_blob_journeys::checkpoint_capture_envelope::checkpoint_capture_child_role",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("WORTH_STORE_C11_BLOB_CHILD_ROLE", "checkpoint-envelope")
        .env("WORTH_STORE_C11_BLOB_CHILD_ROOT", &root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains(REPORT),
        "envelope child failed
stdout:
{stdout}
stderr:
{}",
        String::from_utf8_lossy(&output.stderr)
    );
    if let Some(line) = stdout.lines().find(|line| line.contains(REPORT)) {
        eprintln!("{line}");
    }
}

#[test]
fn checkpoint_capture_child_role() {
    if std::env::var("WORTH_STORE_C11_BLOB_CHILD_ROLE").as_deref() != Ok("checkpoint-envelope") {
        return;
    }
    let root = std::path::PathBuf::from(
        std::env::var_os("WORTH_STORE_C11_BLOB_CHILD_ROOT").expect("checkpoint child root"),
    );
    worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::child(&root);
}
