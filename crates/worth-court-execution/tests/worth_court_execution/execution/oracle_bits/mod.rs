#[allow(unused_imports)]
use crate::oracle_bits::ordinary_context::*;

#[test]
fn canonical_encoding_preserves_floating_point_bits_and_field_boundaries() {
    assert_ne!(bits(&0.0_f64), bits(&-0.0_f64));

    let nan = f64::from_bits(0x7ff8_0000_0000_0042);
    let other_nan = f64::from_bits(0x7ff8_0000_0000_0043);
    assert_eq!(bits(&nan), bits(&nan));
    assert_ne!(bits(&nan), bits(&other_nan));

    assert_ne!(
        bits(&("a".to_owned(), "bc".to_owned())),
        bits(&("ab".to_owned(), "c".to_owned())),
    );
}
