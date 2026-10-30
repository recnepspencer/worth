//! The first recovery publishes a V3 drop without folding its WAL into a
//! checkpoint. A second, independent C8 process must preserve that result.

#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_released_crash::repeat_c8::*;

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_reclaim_released_crash::ROLE,
};

#[test]
fn second_fresh_recovery_preserves_the_first_published_v3_drop() {
    let world = kill_at(ROLE, std::time::Duration::from_secs(240));
    let checkpoint = std::fs::read(world.root.join("families/checkpoint.current")).unwrap();
    let (source, format) =
        crate::blob_reclaim_released_crash::retirement_witness::selected_root(&world.root);
    let (publication, _) =
        crate::blob_reclaim_released_crash::retirement_witness::selected_publication_extent(
            &world.root,
            &source,
        );
    recover_closed_store(&world.root);
    let (first, first_format) =
        crate::blob_reclaim_released_crash::retirement_witness::selected_root(&world.root);
    assert_eq!(first_format, format);
    let first_placements =
        crate::blob_reclaim_released_crash::retirement_witness::selected_placements(
            &world.root,
            &first,
        );
    assert_eq!(first.generation(), source.generation() + 1);
    assert!(first_placements
        .iter()
        .all(|placement| placement.record() != publication));
    assert_eq!(
        std::fs::read(world.root.join("families/checkpoint.current")).unwrap(),
        checkpoint
    );
    let output = run_recovery(&world.root);
    assert!(
        output.status.success(),
        "repeat C8 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("C8_RECOVERY_RUNTIME "),
        "fresh C8 did not construct a recovered runtime: {stderr}"
    );
    assert!(
        stderr.contains("C8_RECOVERY_REDO apply=0 "),
        "repeat must not apply WAL: {stderr}"
    );
    let skip = stderr
        .lines()
        .find_map(|line| line.strip_prefix("C8_RECOVERY_REDO apply=0 skip_historical_drop="))
        .and_then(|count| count.parse::<u64>().ok())
        .expect("bounded historical skip counter");
    assert!(
        skip > 0,
        "the old dropped WAL target must be classified: {stderr}"
    );
    assert!(
        stderr.contains("fate=HistoricalReleaseConsumed"),
        "old WAL group must be terminally consumed: {stderr}"
    );
    let (second, second_format) =
        crate::blob_reclaim_released_crash::retirement_witness::selected_root(&world.root);
    assert_eq!(second_format, format);
    assert_eq!(
        second, first,
        "repeat C8 must not resurrect a dropped WAL target"
    );
    assert_eq!(
        crate::blob_reclaim_released_crash::retirement_witness::selected_placements(
            &world.root,
            &second
        ),
        first_placements
    );
    assert_eq!(
        std::fs::read(world.root.join("families/checkpoint.current")).unwrap(),
        checkpoint
    );
}

#[test]
fn selected_v3_witness_changed_after_first_recovery_denies_repeat() {
    let world = kill_at(ROLE, std::time::Duration::from_secs(240));
    recover_closed_store(&world.root);
    mutate_selected_descriptor_custody(&world.root);
    let output = run_recovery(&world.root);
    assert!(
        !output.status.success(),
        "changed selected V3 custody must deny repeat C8"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("C8_RECOVERY_BLOCKED cause=Damage(PageAdmission)"),
        "changed selected control must fail at historical page admission: {stderr}"
    );
}
