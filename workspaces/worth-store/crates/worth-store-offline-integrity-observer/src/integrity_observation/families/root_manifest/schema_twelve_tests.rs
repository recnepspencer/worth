//! The independent reader accepts a root with the full schema-12 table.
use super::*;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, ManifestBlockReference, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, RecordAnchorAdvance, RecordAnchorName, RecordAnchorTable,
};
#[test]
fn offline_observer_reads_schema_twelve_and_rejects_eleven_without_decoding() {
    let record = |i| PersistedRecordIdentity::new([7; 16], i).unwrap();
    let mut anchors = RecordAnchorTable::EMPTY;
    for i in 0..32 {
        anchors = anchors
            .advanced(
                RecordAnchorAdvance::new(
                    RecordAnchorName::new(&format!("owner.{i:02}")).unwrap(),
                    0,
                    record(i + 1),
                    [i as u8; 32],
                )
                .unwrap(),
            )
            .unwrap();
    }
    let root = DurablePhysicalRootManifest::builder(1, 9, 2, 43)
        .record_count(32)
        .next_block(2)
        .routing_root(Some(
            ManifestBlockReference::new(1, 1, 4, 31, record(1), record(32)).unwrap(),
        ))
        .record_anchors(anchors)
        .admit()
        .unwrap();
    let mut bytes = root.encode(PhysicalRecordFormatDeclaration::builder().admit().unwrap());
    assert_eq!(bytes[9], 12);
    assert_eq!(bytes.len(), 3808);
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(read_root_manifest(&bytes, &mut counters).is_ok());
    assert_eq!(counters.root_manifest_payload_decoder_entries, 1);
    bytes[9] = 11;
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(matches!(
        read_root_manifest(&bytes, &mut counters),
        Err(OfflineIntegrityOutcome::Unsupported(_))
    ));
    assert_eq!(counters.root_manifest_payload_decoder_entries, 0);
}
