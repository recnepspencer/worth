use super::BoundedMediaWalk;
use crate::{
    OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationLimits, OfflineIntegrityOutcome,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[test]
fn cached_snapshot_does_not_reconsume_bytes_or_hide_later_source_change() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-cached-source-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = std::fs::canonicalize(root).unwrap();
    let path = root.join("artifact");
    std::fs::write(&path, b"before!!").unwrap();
    let limits = OfflineIntegrityObservationLimits::new(8, 4096, 5, 4, 0, 10_000, 4096).unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    assert_eq!(walk.acquire(&path, 1).unwrap().bytes.as_ref(), b"before!!");
    assert_eq!(walk.acquire(&path, 1).unwrap().bytes.as_ref(), b"before!!");
    assert_eq!(walk.counters_mut().bytes_read, 8);
    std::fs::write(&path, b"changed-length").unwrap();
    assert_eq!(
        walk.acquire(&path, 1).unwrap_err(),
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    );
    assert_eq!(walk.counters_mut().bytes_read, 8);
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&root).unwrap();
}
