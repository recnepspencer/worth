//! Stable retained-preimage vocabulary evidence.

use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

use super::retained_preimage::demanded_field_slot;

/// A demand has no nested-path vocabulary, so only one exact segment can name
/// a demanded field slot.
#[test]
fn only_a_single_segment_path_names_a_demanded_field_slot() {
    let exact = CanonicalFieldPath::single(FieldKey::new("Status").unwrap());
    assert_eq!(
        demanded_field_slot(&exact).map(FieldKey::as_str),
        Some("Status")
    );

    let nested = CanonicalFieldPath::new([
        FieldKey::new("Account").unwrap(),
        FieldKey::new("Status").unwrap(),
    ])
    .unwrap();
    assert!(
        demanded_field_slot(&nested).is_none(),
        "a nested path must not satisfy a demand through its first segment"
    );
}
