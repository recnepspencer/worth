//! Pure wire framing and shared canonical leaf emission.
use super::{framing_length, write_scalar, RelationalHomeProgramBodyDenial};
use crate::aspect_wire::{write_aspect_field_locator, write_aspect_value, CollectCanonicalAspect};
use crate::transactions::data::planned_single_field_locator;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InlineBytes};

#[test]
fn body_framing_refuses_unrepresentable_length() {
    if let Ok(length) = usize::try_from(u64::from(u32::MAX) + 1) {
        assert!(matches!(
            framing_length(length),
            Err(RelationalHomeProgramBodyDenial::CapacityOverflow)
        ));
    }
}
#[test]
fn streamed_leaf_commitments_keep_the_preexisting_canonical_wire_bytes() {
    for (value, golden) in [
        (
            AspectValue::InlineBytes(InlineBytes::try_new(vec![0, 128, 255]).unwrap()),
            vec![24, 3, 0, 0, 0, 0, 128, 255],
        ),
        (
            AspectValue::Int64(-12),
            vec![5, 244, 255, 255, 255, 255, 255, 255, 255],
        ),
    ] {
        let mut bytes = Vec::new();
        let mut out = CollectCanonicalAspect(&mut bytes);
        write_aspect_value(&mut out, &value).unwrap();
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(&bytes)),
            <[u8; 32]>::from(Sha256::digest(&golden))
        );
        assert_eq!(crate::aspect_wire::encode_aspect_value(&value), golden);
    }
    let locator =
        planned_single_field_locator(AspectKey::new("a").unwrap(), FieldKey::new("b").unwrap());
    let golden = [2, 5, 1, 0, 0, 0, b'a', 1, 0, 0, 0, 1, 0, 0, 0, b'b'];
    let mut bytes = Vec::new();
    let mut out = CollectCanonicalAspect(&mut bytes);
    write_aspect_field_locator(&mut out, &locator).unwrap();
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(&bytes)),
        <[u8; 32]>::from(Sha256::digest(golden))
    );
    assert_eq!(
        crate::aspect_wire::encode_aspect_field_locator(&locator),
        golden
    );
}

#[test]
fn scalar_framing_distinguishes_embedded_nul_from_separate_values() {
    let mut together = Vec::new();
    write_scalar(&mut together, &"a\0b").unwrap();
    let mut separate = Vec::new();
    write_scalar(&mut separate, &"a").unwrap();
    write_scalar(&mut separate, &"b").unwrap();
    assert_ne!(together, separate);
}

#[test]
fn authored_body_policy_refuses_live_materialization() {
    let materialization = crate::mvcc::RelationalTransactionIntent::materialization(
        crate::mvcc::RelationalMaterializationTransactionMode::Suspend,
    );
    assert!(matches!(
        super::authored_policy(&materialization),
        Err(RelationalHomeProgramBodyDenial::UnsupportedMaterialization)
    ));
}
