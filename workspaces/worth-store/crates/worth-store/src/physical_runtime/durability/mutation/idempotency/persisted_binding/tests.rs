use super::super::canonical_encoding::CanonicalBindingComparison;
use super::*;

pub(in crate::physical_runtime::durability::mutation::idempotency) mod fixture;

#[test]
fn persisted_decode_retains_exact_validated_bytes_and_rejects_trailing_input() {
    let (fixture, binding) = fixture::binding();
    let bytes = binding.bytes();
    let context =
        PhysicalBindingDecodingContext::new(fixture.store, fixture.policy, fixture.idempotency);
    let decoded =
        PersistedPhysicalMutationAttemptBinding::decode_from_compaction(bytes, context).unwrap();
    assert_eq!(&decoded, &binding);
    assert_eq!(decoded.bytes.len(), bytes.len());
    let mut comparison = CanonicalBindingComparison::new(bytes);
    binding.encode_into(&mut comparison);
    assert!(comparison.matches());
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert_eq!(
        PersistedPhysicalMutationAttemptBinding::decode_from_compaction(&trailing, context),
        Err(PhysicalPersistedBindingDecodeDenial::TrailingBytes),
    );
    fixture.media.close();
}
