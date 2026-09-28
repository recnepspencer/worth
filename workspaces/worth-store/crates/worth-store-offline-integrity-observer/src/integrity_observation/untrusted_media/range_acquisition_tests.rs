use super::BoundedMediaWalk;
use crate::{
    OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationLimits, OfflineIntegrityOutcome,
};
use std::io::{Seek, SeekFrom, Write};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[test]
fn arena_ranges_charge_only_observed_bytes_and_reject_changed_source() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("worth-arena-range-{}-{unique}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let root = std::fs::canonicalize(root).unwrap();
    let path = root.join("arena");
    let mut file = std::fs::File::create(&path).unwrap();
    file.set_len(16 * 1024 * 1024).unwrap();
    file.seek(SeekFrom::Start(1024 * 1024)).unwrap();
    file.write_all(b"routed bytes").unwrap();
    drop(file);
    let limits = OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    let observed = walk.acquire_range(&path, 1, 1024 * 1024, 12).unwrap();
    assert_eq!(observed.bytes.as_ref(), b"routed bytes");
    assert_eq!(observed.byte_length, 16 * 1024 * 1024);
    assert_eq!(walk.counters_mut().bytes_read, 12);
    assert!(walk
        .seen_files
        .values()
        .all(|cached| cached.bytes.is_none()));
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(1024)
        .unwrap();
    assert_eq!(
        walk.acquire_range(&path, 1, 0, 12).unwrap_err(),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert_eq!(walk.counters_mut().bytes_read, 12);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(root).unwrap();
}

#[cfg(windows)]
#[test]
fn arena_range_rejects_same_length_path_replacement() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-arena-replacement-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = std::fs::canonicalize(root).unwrap();
    let path = root.join("arena");
    std::fs::write(&path, b"old frame").unwrap();
    let limits = OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    assert_eq!(
        walk.acquire_range(&path, 1, 0, 9).unwrap().bytes.as_ref(),
        b"old frame"
    );

    let displaced = root.join("displaced");
    std::fs::rename(&path, &displaced).unwrap();
    std::fs::write(&path, b"new frame").unwrap();
    assert_eq!(
        walk.acquire_range(&path, 1, 0, 9).unwrap_err(),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert_eq!(walk.counters_mut().bytes_read, 9);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(displaced).unwrap();
    std::fs::remove_dir(root).unwrap();
}
