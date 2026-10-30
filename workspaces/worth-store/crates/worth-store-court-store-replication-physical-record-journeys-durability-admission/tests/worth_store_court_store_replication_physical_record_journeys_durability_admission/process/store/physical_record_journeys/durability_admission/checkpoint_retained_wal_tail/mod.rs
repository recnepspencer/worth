#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::*;

#[test]
fn published_checkpoint_carries_the_exact_original_multi_rotation_wal_tail() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement) = rotation_world(&store_root);
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(1, b"checkpoint-base")]);
    let (completed, frozen_source) =
        publish_after_foreground_rotations(&serving, submission, placement);
    assert_independent_tail(&store_root, frozen_source, &completed);
    assert_reclaimed_prefix(&completed, 1);

    serving.close();
}

#[test]
fn foreground_wal_cannot_cross_the_checkpoint_publication_cutover() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement) = rotation_world(&store_root);
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(31, b"cutover-base")]);
    let (gate, handle, publication_arrival) = checkpoint_at_publication_replacement(&serving);
    let frozen_source = handle.source();
    let (foreground, completion) = start_cutover_foreground(submission, placement);
    assert!(matches!(
        completion.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
    let completed = finish_checkpoint_cutover(&gate, handle, publication_arrival, &completion);
    assert!(matches!(
        completion.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));

    let tail = completed.retained_wal_tail();
    let boundary = frozen_source.wal().covered_end_lsn_exclusive();
    assert_eq!(tail.checkpoint_boundary_lsn().get(), boundary);
    assert_eq!(tail.durable_tail_end_lsn_exclusive().get(), boundary);
    assert_eq!(tail.segments().len(), 1);
    assert!(matches!(
        completed.wal_reclamation(),
        worth_store::physical_runtime::PhysicalWalReclamationObservation::NotRequired { .. }
    ));

    gate.release();
    let foreground_end = completion.recv().unwrap();
    foreground.join().unwrap();
    assert!(foreground_end > tail.durable_tail_end_lsn_exclusive().get());
    serving.close();
}
