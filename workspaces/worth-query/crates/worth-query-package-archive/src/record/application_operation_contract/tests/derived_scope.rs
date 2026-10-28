//! Composite scope5 is an explicit versioned association, not leaf permission.
use super::*;
use worth_query_installation::facade::WorthQueryPortableOperationGraphReadScope as Scope;
#[test]
fn derived_source_scope_is_versioned_and_unique_in_canonical_order() {
    let scope = Scope::DerivedSource {
        schema: "Graph".into(),
        relation: "Adjacency".into(),
    };
    let mut output = BinaryOutput::with_capacity(64);
    super::super::graph_read::write(&mut output, &scope).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..4], &[0, 5, 0, 1]);
    let mut budget = RecordDecodeAttempt::begin(
        WorthQueryPackageArchiveDecodeWork::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        super::super::graph_read::decode(&mut input, &mut budget).unwrap(),
        scope
    );
    assert!(input.is_finished());
    let mut future = bytes.clone();
    future[3] = 2;
    let mut input = BinaryInput::new(&future[..4]);
    let denial = super::super::graph_read::decode(&mut input, &mut budget).unwrap_err();
    assert_eq!(denial.kind(), Kind::UnsupportedRecordVariant);
    let mut parts = fixture::complete_parts();
    parts.graph_reads.push(scope.clone());
    let record = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap().record(),
        &fixture::wrapped(record)
    );
    let mut parts = fixture::complete_parts();
    parts.graph_reads.extend([scope.clone(), scope]);
    let duplicate =
        WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&duplicate, WorthQueryPackageArchiveLimits::DEFAULT);
    assert!(decoder.decode_frame(&bytes).is_err());
}
