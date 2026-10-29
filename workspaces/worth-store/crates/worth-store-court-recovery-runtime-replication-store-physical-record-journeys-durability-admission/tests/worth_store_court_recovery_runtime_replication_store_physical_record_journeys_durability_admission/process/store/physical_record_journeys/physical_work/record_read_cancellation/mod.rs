#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::record_read_cancellation::*;
use {worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};

use {worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::read_record, worth_store_testkit_store::physical_record_journeys::serving_from_open};
use {worth_store_testkit_store::physical_work::record_read_signal_cleanup::await_read_signal_cleanup};
use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

#[test]
fn cancelling_a_read_session_reports_unread_delivery_and_releases_its_leases() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &initial,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("physical-work-read-cancellation", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let serving = serving_from_open(&root);
    let observer = serving.observer();
    let limits = worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
    let mut session = serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
        .unwrap();
    await_read_signal_cleanup(&serving);
    assert_eq!(observer.record_counters().read_sessions_live(), 1);
    let mut prefix = [0_u8; 5];
    assert_eq!(session.read_next(&mut prefix).unwrap(), prefix.len());
    assert_eq!(&prefix, &PAYLOAD[..prefix.len()]);
    let media_before_cancel = serving.media_counters();

    let cancellation = session.cancel();

    assert_eq!(cancellation.observation().bytes_completed(), 5);
    assert_eq!(
        cancellation.unread_payload_bytes(),
        (PAYLOAD.len() - prefix.len()) as u64
    );
    assert!(!cancellation.delivery_was_complete());
    assert!(cancellation.observation().physical_work_count() > 0);
    assert_eq!(observer.record_counters().read_sessions_live(), 0);
    assert_eq!(serving.media_counters(), media_before_cancel);
    let (bytes, _) = read_record(
        serving
            .records()
            .expect("read protection admission")
            .open(record, limits)
            .unwrap(),
        PAYLOAD.len(),
    );
    assert_eq!(bytes, PAYLOAD);
    await_read_signal_cleanup(&serving);
    assert!(!serving.close_plan().execute().requires_inspection());
}
