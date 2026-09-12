use worth_store::physical_runtime::{
    PhysicalWorkCounterStage, RecordAppendBatch, RecordByteLimit, RecordReadDenial,
    RecordReadLimits,
};
use worth_store_physical_backend::{
    ArtifactTreeFailureKind, MediaFaultDirective, MediaOperationRole,
};

use super::super::{durable_publication, serving_from_open};
use super::record_read_signal_cleanup::await_read_signal_cleanup;
use super::{configuration, serving_from_initialization};

const PAYLOAD: &[u8] = b"canonical read failure cleanup";

#[test]
fn denied_before_effect_read_releases_work_and_allows_same_runtime_retry() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = durable_publication::publish_single(
        &initial,
        placement,
        durable_publication::certification_material("record-read-failure-cleanup", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let calibration = serving_from_open(&root);
    let bootstrap_reads = calibration
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedRead);
    calibration.close();

    let serving = super::fault_fixture::serving_from_open_with_positioned_read_fault(
        &root,
        bootstrap_reads + 1,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let limits = RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
    let before_media = serving.media_counters();
    let before_terminal = serving
        .physical_work_counters()
        .total(PhysicalWorkCounterStage::Terminal);
    let invalidations_before = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let failure = match serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
    {
        Ok(_) => panic!("a denied backend read must not construct a record session"),
        Err(failure) => failure,
    };

    assert!(matches!(
        failure.denial(),
        RecordReadDenial::BackendUnavailable(failure)
            if failure.kind() == ArtifactTreeFailureKind::DeniedBeforeEffect
    ));
    assert!(failure.observation().physical_work_count() > 0);
    await_read_signal_cleanup(&serving);
    let after_media = serving.media_counters();
    assert_eq!(
        serving
            .physical_signal_observation()
            .unwrap()
            .aspect_invalidation_count(),
        invalidations_before,
        "a transient denial must not invalidate physical projection truth"
    );
    assert_eq!(
        after_media.attempts_for(MediaOperationRole::PositionedRead)
            - before_media.attempts_for(MediaOperationRole::PositionedRead),
        1
    );
    assert_eq!(
        after_media.completed_bytes_for(MediaOperationRole::PositionedRead)
            - before_media.completed_bytes_for(MediaOperationRole::PositionedRead),
        0
    );
    assert!(
        serving
            .physical_work_counters()
            .total(PhysicalWorkCounterStage::Terminal)
            > before_terminal,
        "dropping the failed public read must terminally release retry-pending work"
    );
    let mut retry = serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
        .expect("transient backend denial must not revoke healthy Store truth");
    let mut bytes = vec![0_u8; PAYLOAD.len()];
    assert_eq!(retry.read_next(&mut bytes).unwrap(), PAYLOAD.len());
    assert_eq!(bytes, PAYLOAD);
    drop(retry);
    assert!(!serving.close_plan().execute().requires_inspection());
}
