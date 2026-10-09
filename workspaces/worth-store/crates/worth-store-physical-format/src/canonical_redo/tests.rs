use super::*;

/// Storage that refuses every request and remembers the last one.
struct Deny {
    requested: Option<u64>,
}

impl PhysicalRecoveryDecodeStorage for Deny {
    type Denial = &'static str;
    fn admit_allocation(&mut self, bytes: u64) -> Result<(), Self::Denial> {
        self.requested = Some(bytes);
        Err("recovery pool exhausted")
    }
}

#[test]
fn truncated_in_policy_count_rejects_before_any_backing_request() {
    let mut bytes = Vec::new();
    field(&mut bytes, CANONICAL_REDO_V3_DOMAIN);
    bytes.extend_from_slice(&4_u64.to_le_bytes());
    let mut storage = Deny { requested: None };
    let result =
        decode_canonical_redo_v3_with_storage(&bytes, 1, 5, 4, limits(4), format(), &mut storage);
    assert!(matches!(
        result,
        Err(PhysicalRecoveryDecodeFailure::Canonical(
            CanonicalRedoWireDenial::MalformedMember
        ))
    ));
    assert_eq!(storage.requested, None);
    let target_count_only = 4_u64.to_le_bytes();
    let mut cursor = Cursor::new(&target_count_only);
    let targets = decode_targets(&mut cursor, &mut 0, 4, &mut None, &mut storage);
    assert!(matches!(
        targets,
        Err(PhysicalRecoveryDecodeFailure::Canonical(
            CanonicalRedoWireDenial::MalformedMember
        ))
    ));
    assert_eq!(storage.requested, None);
}

/// The least bytes one record and one target occupy on the wire.
const RECORD_BYTES: usize = 4 + 8 + 8 + 8;
const TARGET_BYTES: usize = 8 + 32 + 1;

#[test]
fn count_limit_precedes_record_allocation() {
    // Five records the member can hold, of the one target admitted.
    let mut bytes = Vec::new();
    field(&mut bytes, CANONICAL_REDO_V3_DOMAIN);
    bytes.extend_from_slice(&5_u64.to_le_bytes());
    let unbacked = bytes.len() + 5 * RECORD_BYTES - 1;
    bytes.resize(unbacked + 1, 0);
    let mut storage = Deny { requested: None };
    let mut decode = |bytes: &[u8]| {
        decode_canonical_redo_v3_with_storage(bytes, 1, 6, 1, limits(1), format(), &mut storage)
            .map(|_| ())
    };
    assert!(matches!(
        decode(&bytes),
        Err(PhysicalRecoveryDecodeFailure::Canonical(
            CanonicalRedoWireDenial::TargetLimit {
                observed: 5,
                admitted: 1
            }
        ))
    ));
    // One byte short of holding them, the count is no count of this member:
    // damage, whatever the limit is.
    assert!(matches!(
        decode(&bytes[..unbacked]),
        Err(PhysicalRecoveryDecodeFailure::Canonical(
            CanonicalRedoWireDenial::MalformedMember
        ))
    ));
    assert_eq!(storage.requested, None);
}

#[test]
fn targets_past_the_limit_name_the_total_that_passed_it() {
    // Two targets came before. Three more are five of the four admitted,
    // refused before any is read or backed.
    let mut bytes = 3_u64.to_le_bytes().to_vec();
    bytes.resize(8 + 3 * TARGET_BYTES, 0);
    let mut storage = Deny { requested: None };
    let mut decode = |bytes: &[u8]| {
        decode_targets(&mut Cursor::new(bytes), &mut 2, 4, &mut None, &mut storage).map(|_| ())
    };
    assert!(matches!(
        decode(&bytes),
        Err(PhysicalRecoveryDecodeFailure::Canonical(
            CanonicalRedoWireDenial::TargetLimit {
                observed: 5,
                admitted: 4
            }
        ))
    ));
    // One byte short of holding three targets, the count is damage.
    assert!(matches!(
        decode(&bytes[..bytes.len() - 1]),
        Err(PhysicalRecoveryDecodeFailure::Canonical(
            CanonicalRedoWireDenial::MalformedMember
        ))
    ));
    assert_eq!(storage.requested, None);
}

#[test]
fn distinct_limit_rejects_second_identity_before_retention() {
    let mut distinct = BTreeSet::new();
    let bytes = encoded_record(&[(1, [1; 32]), (2, [2; 32])]);
    assert_eq!(
        decode_canonical_redo_v3(
            &bytes,
            1,
            2,
            2,
            Some((&mut distinct, 1)),
            limits(2),
            format(),
        ),
        Err(CanonicalRedoWireDenial::DistinctTargetLimit {
            observed: 2,
            admitted: 1
        })
    );
    assert_eq!(distinct.len(), 1);
}

#[test]
fn canonical_member_preserves_nested_unsupported_projection_version() {
    let mut bytes = encoded_record(&[(1, [1; 32])]);
    let mut projection = Vec::new();
    field(&mut projection, b"store.physical.recovery-projection.v14");
    field(&mut bytes, &projection);
    assert_eq!(
        decode_canonical_redo_v3(&bytes, 1, 2, 1, None, limits(1), format()),
        Err(CanonicalRedoWireDenial::UnsupportedRecoveryProjectionVersion(14))
    );
}

fn limits(maximum: u64) -> PhysicalRecoveryProjectionDecodeLimits {
    PhysicalRecoveryProjectionDecodeLimits {
        frames: maximum,
        record_identities: maximum,
        placements: maximum,
        segment_updates: maximum,
        manifests: maximum,
        total_entries: maximum.saturating_mul(3),
        inline_allocations: maximum,
    }
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn encoded_record(targets: &[(u64, [u8; 32])]) -> Vec<u8> {
    let mut encoded = Vec::new();
    field(&mut encoded, CANONICAL_REDO_V3_DOMAIN);
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
