use std::fs;
#[cfg(not(windows))]
use std::thread;
#[cfg(not(windows))]
use std::time::Duration;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use super::BoundedMediaWalk;
use crate::{
    OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationLimits, OfflineIntegrityOutcome,
};

#[cfg(not(windows))]
#[test]
fn same_length_change_after_read_is_indeterminate() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-offline-source-change-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&root).expect("temporary root");
    let path = root.join("artifact");
    fs::write(&path, b"before!!").expect("initial file");
    let limits =
        OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).expect("limits");
    let canonical_root = fs::canonicalize(&root).expect("canonical root");
    let path = canonical_root.join("artifact");
    let mut walk = BoundedMediaWalk::new(limits, canonical_root, Instant::now());
    let outcome = walk.acquire_with_after_read(&path, 1, || {
        thread::sleep(Duration::from_millis(20));
        fs::write(&path, b"after!!!").expect("same-length mutation");
    });
    assert_eq!(
        outcome.expect_err("changing source must be refused"),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    fs::remove_dir_all(&root).expect("remove temporary root");
}

#[cfg(unix)]
#[test]
fn atomic_path_replacement_after_read_is_indeterminate() {
    let (root, _path, limits) = unix_fixture("atomic-replacement");
    let displaced = root.join("displaced");
    let canonical_root = fs::canonicalize(&root).expect("canonical root");
    let path = canonical_root.join("artifact");
    let mut walk = BoundedMediaWalk::new(limits, canonical_root, Instant::now());
    let outcome = walk.acquire_with_after_read(&path, 1, || {
        fs::rename(&path, &displaced).expect("displace observed inode");
        fs::write(&path, b"after!!!").expect("install same-length replacement");
    });
    assert_eq!(
        outcome.expect_err("pathname replacement must be refused"),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    fs::remove_dir_all(&root).expect("remove temporary root");
}

#[cfg(unix)]
#[test]
fn pre_open_escaping_symlink_swap_is_refused_before_read() {
    use std::os::unix::fs::symlink;

    let (root, _path, limits) = unix_fixture("pre-open-symlink");
    let external = root
        .parent()
        .expect("temporary parent")
        .join("external-artifact");
    fs::write(&external, b"outside!").expect("external target");
    let canonical_root = fs::canonicalize(&root).expect("canonical root");
    let path = canonical_root.join("artifact");
    let mut walk = BoundedMediaWalk::new(limits, canonical_root, Instant::now());
    let outcome = walk.acquire_with_before_open(&path, 1, || {
        fs::remove_file(&path).expect("remove admitted path");
        symlink(&external, &path).expect("install escaping symlink");
    });
    assert_eq!(
        outcome.expect_err("pre-open symlink substitution must be refused"),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert_eq!(walk.counters_mut().bytes_read, 0);
    fs::remove_file(&external).expect("remove external target");
    fs::remove_dir_all(&root).expect("remove temporary root");
}

#[cfg(unix)]
#[test]
fn unknown_parent_symlink_swap_is_indeterminate_before_identity_registration() {
    use std::os::unix::fs::symlink;

    let (root, _, limits) = unix_fixture("unknown-parent-symlink");
    let records = root.join("records");
    fs::create_dir(&records).expect("records directory");
    let unknown = records.join("mystery.record");
    fs::write(&unknown, b"inside!!").expect("inside unknown");
    let external = root.parent().expect("temporary parent").join(format!(
        "external-{}",
        root.file_name()
            .expect("temporary root name")
            .to_string_lossy()
    ));
    fs::create_dir(&external).expect("external directory");
    fs::write(external.join("mystery.record"), b"outside!").expect("external unknown");
    let displaced = root.join("displaced-records");
    let mut walk = BoundedMediaWalk::new(
        limits,
        fs::canonicalize(&root).expect("canonical root"),
        Instant::now(),
    );
    let classification = walk.classify_unrecognized_with_before_open(&unknown, || {
        fs::rename(&records, &displaced).expect("displace admitted parent");
        symlink(&external, &records).expect("install escaping parent symlink");
    });
    assert_eq!(
        classification.outcome,
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert!(classification.physical_alias_of.is_none());
    assert_eq!(walk.counters_mut().bytes_read, 0);
    fs::remove_file(&records).expect("remove parent symlink");
    fs::remove_dir_all(&external).expect("remove external directory");
    fs::remove_dir_all(&root).expect("remove temporary root");
}

#[cfg(unix)]
fn unix_fixture(
    label: &str,
) -> (
    std::path::PathBuf,
    std::path::PathBuf,
    OfflineIntegrityObservationLimits,
) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-offline-{label}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&root).expect("temporary root");
    let path = root.join("artifact");
    fs::write(&path, b"before!!").expect("initial file");
    let limits =
        OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).expect("limits");
    (root, path, limits)
}

#[cfg(windows)]
#[test]
fn live_observer_handle_denies_same_length_replacement() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-offline-source-lock-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&root).expect("temporary root");
    let path = root.join("artifact");
    fs::write(&path, b"before!!").expect("initial file");
    let limits =
        OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).expect("limits");
    let canonical_root = fs::canonicalize(&root).expect("canonical root");
    let path = canonical_root.join("artifact");
    let mut walk = BoundedMediaWalk::new(limits, canonical_root, Instant::now());
    let mut replacement_denied = false;
    let acquired = walk
        .acquire_with_after_read(&path, 1, || {
            replacement_denied = fs::write(&path, b"after!!!").is_err();
        })
        .expect("locked source remains stable");
    assert!(replacement_denied, "live handle must deny replacement");
    assert_eq!(&*acquired.bytes, b"before!!");
    fs::remove_dir_all(&root).expect("remove temporary root");
}

#[cfg(windows)]
#[test]
fn final_symlink_swap_is_refused_before_windows_read() {
    use std::os::windows::fs::symlink_file;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-offline-final-link-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&root).expect("temporary root");
    let initial = root.join("artifact");
    fs::write(&initial, b"inside!!").expect("initial file");
    let external = root.parent().expect("temporary parent").join(format!(
        "external-final-link-{}-{unique}",
        std::process::id()
    ));
    fs::write(&external, b"outside!").expect("external target");
    let canonical_root = fs::canonicalize(&root).expect("canonical root");
    let path = canonical_root.join("artifact");
    let limits =
        OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).expect("limits");
    let mut walk = BoundedMediaWalk::new(limits, canonical_root, Instant::now());
    let outcome = walk.acquire_with_before_open(&path, 1, || {
        fs::remove_file(&path).expect("remove admitted file");
        symlink_file(&external, &path).expect("install final symlink");
    });
    assert_eq!(
        outcome.expect_err("final symlink substitution must be refused"),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert_eq!(walk.counters_mut().bytes_read, 0);
    fs::remove_file(&external).expect("remove external target");
    fs::remove_dir_all(&root).expect("remove temporary root");
}

#[cfg(windows)]
#[test]
fn unknown_parent_symlink_swap_is_refused_before_windows_registration() {
    use std::os::windows::fs::symlink_dir;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-offline-parent-link-{}-{unique}",
        std::process::id()
    ));
    let records = root.join("records");
    fs::create_dir_all(&records).expect("records directory");
    fs::write(records.join("mystery.record"), b"inside!!").expect("inside unknown");
    let external = root.parent().expect("temporary parent").join(format!(
        "external-parent-link-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&external).expect("external directory");
    fs::write(external.join("mystery.record"), b"outside!").expect("external unknown");
    let canonical_root = fs::canonicalize(&root).expect("canonical root");
    let records = canonical_root.join("records");
    let unknown = records.join("mystery.record");
    let displaced = canonical_root.join("displaced-records");
    let limits =
        OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).expect("limits");
    let mut walk = BoundedMediaWalk::new(limits, canonical_root, Instant::now());
    let classification = walk.classify_unrecognized_with_before_open(&unknown, || {
        fs::rename(&records, &displaced).expect("displace admitted parent");
        symlink_dir(&external, &records).expect("install parent symlink");
    });
    assert_eq!(
        classification.outcome,
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert!(classification.physical_alias_of.is_none());
    assert_eq!(walk.counters_mut().bytes_read, 0);
    fs::remove_dir(&records).expect("remove parent symlink");
    fs::remove_dir_all(&external).expect("remove external directory");
    fs::remove_dir_all(&root).expect("remove temporary root");
}
