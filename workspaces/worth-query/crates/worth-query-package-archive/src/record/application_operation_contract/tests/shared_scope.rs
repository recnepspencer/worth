//! Shared references preserve the complete schema/content pair and strict wire order.
use super::*;
use worth_query_installation::facade::{
    WorthQueryPortableApplicationOperationContractRecord as Record,
    WorthQueryPortableOperationGraphReadScope as Scope,
};

#[test]
fn shared_graph_scope_pairs_roundtrip_and_refuse_duplicate_or_descending_wire() {
    let scope = |schema: &str, identity: &str| Scope::SharedDecisionRead {
        schema: schema.into(),
        identity: identity.into(),
    };
    let ordered = [scope("A", "a"), scope("A", "b"), scope("B", "a")];
    let mut parts = fixture::complete_parts();
    parts.graph_reads.extend(ordered.clone());
    let record = Record::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap().record(),
        &fixture::wrapped(record)
    );
    for pair in [
        vec![ordered[0].clone(), ordered[0].clone()],
        vec![ordered[1].clone(), ordered[0].clone()],
        vec![ordered[2].clone(), ordered[0].clone()],
    ] {
        let mut bad = fixture::complete_parts();
        bad.graph_reads.extend(pair);
        let bytes = encode_untrusted(
            &Record::from_untrusted_parts(bad),
            WorthQueryPackageArchiveLimits::DEFAULT,
        );
        assert_eq!(
            decoder.decode_frame(&bytes).unwrap_err().kind(),
            Kind::NonCanonicalRecordSequence
        );
    }
    let mut budget = RecordDecodeAttempt::begin(
        WorthQueryPackageArchiveDecodeWork::default(),
        4,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(
        super::super::graph_read::decode(&mut BinaryInput::new(&[0, 6, 0, 2]), &mut budget)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVariant
    );
}
