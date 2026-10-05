use super::*;

#[test]
fn record_count_is_rejected_before_record_vector_allocation() {
    let range = WalLsnRange::new(LogSequenceNumber::new(1), LogSequenceNumber::new(6)).unwrap();
    let mut encoded = Vec::new();
    field(&mut encoded, REDO_DOMAIN);
    encoded.extend_from_slice(&5_u64.to_le_bytes());
    // A count the member cannot hold is damage before it is any limit.
    assert_eq!(
        decode_physical_redo_records(&encoded, range, 1, test_format()),
        Err(PhysicalRedoPlanningDenial::MalformedMember)
    );
    // Five records the member can hold, each of four words at the least.
    encoded.resize(encoded.len() + 5 * (4 + 8 + 8 + 8), 0);
    assert_eq!(
        decode_physical_redo_records(&encoded, range, 1, test_format()),
        Err(PhysicalRedoPlanningDenial::TargetLimit {
            observed: 5,
            admitted: 1
        })
    );
}

#[test]
fn new_one_over_distinct_target_is_rejected_before_retention() {
    let range = WalLsnRange::new(LogSequenceNumber::new(1), LogSequenceNumber::new(2)).unwrap();
    let encoded = encoded_record(&[(1, [1; 32]), (2, [2; 32])]);
    let mut distinct = BTreeSet::new();
    assert_eq!(
        decode_physical_redo_records_with_distinct(
            &encoded,
            range,
            2,
            &mut distinct,
            1,
            test_format(),
        ),
        Err(PhysicalRedoPlanningDenial::DistinctTargetLimit {
            observed: 2,
            admitted: 1
        })
    );
    assert_eq!(distinct.len(), 1);
}

fn test_format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn encoded_record(targets: &[(u64, [u8; 32])]) -> Vec<u8> {
    let mut encoded = Vec::new();
    field(&mut encoded, REDO_DOMAIN);
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u32.to_le_bytes());
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&(targets.len() as u64).to_le_bytes());
    for (page, digest) in targets {
        let mut target = Vec::new();
        target.push(1);
        target.extend_from_slice(&1_u64.to_le_bytes());
        target.extend_from_slice(&page.to_le_bytes());
        target.extend_from_slice(&1_u64.to_le_bytes());
        target.push(5);
        target.extend_from_slice(&1_u64.to_le_bytes());
        target.extend_from_slice(&1_u64.to_le_bytes());
        target.extend_from_slice(&0_u64.to_le_bytes());
        target.extend_from_slice(&4096_u32.to_le_bytes());
        field(&mut encoded, &target);
        encoded.extend_from_slice(digest);
    }
    field(&mut encoded, b"redo");
    encoded
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}
