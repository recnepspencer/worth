#[allow(unused_imports)]
use crate::checkpoint_validation::stream_bindings::*;

#[test]
fn literal_records_have_independent_crc_and_sha_vectors() {
    for record in [
        HEADER.as_slice(),
        DIRTY_BASIS.as_slice(),
        BINDING_COMPACTION.as_slice(),
        BINDING.as_slice(),
        FOOTER.as_slice(),
    ] {
        let checksum_offset = record.len() - 4;
        assert_eq!(
            u32::from_le_bytes(record[checksum_offset..].try_into().unwrap()),
            crc32c(&record[..checksum_offset])
        );
    }
    assert_eq!(sha256(&DIRTY_BASIS), DIRTY_AGGREGATE);
    assert_eq!(sha256(&BINDING), BINDING_AGGREGATE);
    assert_eq!(&FOOTER[48..80], DIRTY_AGGREGATE);
    assert_eq!(&FOOTER[120..152], BINDING_AGGREGATE);

    let mut aggregate = worth_store_physical_format::CheckpointSelectiveRecordAggregate::new();
    aggregate.include(&DIRTY_BASIS).unwrap();
    assert_eq!(aggregate.summary().digest(), DIRTY_AGGREGATE);
    assert_eq!(aggregate.summary().record_count(), 1);
    assert_eq!(aggregate.summary().encoded_bytes(), 68);
}
