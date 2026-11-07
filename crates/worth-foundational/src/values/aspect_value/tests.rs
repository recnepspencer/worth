use super::AspectValue;
use crate::values::{CanonicalBigInt, CanonicalDecimal, CanonicalRational, InternedString, Symbol};

#[test]
fn owned_capacity_counts_every_variable_width_scalar_family() {
    let decimal = AspectValue::Decimal(CanonicalDecimal::new(retained("12.5", 64)));
    let big_int = AspectValue::BigInt(CanonicalBigInt::new(retained("123", 96)));
    let rational = AspectValue::Rational(
        CanonicalRational::new(
            CanonicalBigInt::new(retained("7", 128)),
            CanonicalBigInt::new(retained("9", 160)),
        )
        .unwrap(),
    );
    let string = AspectValue::String(InternedString::Raw(retained("text", 192)));

    assert_eq!(decimal.owned_allocation_capacity_bytes(), 64);
    assert_eq!(big_int.owned_allocation_capacity_bytes(), 96);
    assert_eq!(rational.owned_allocation_capacity_bytes(), 288);
    assert_eq!(string.owned_allocation_capacity_bytes(), 192);
}

#[test]
fn inline_scalar_families_report_no_owned_allocation() {
    for value in [
        AspectValue::Null,
        AspectValue::UInt64(7),
        AspectValue::Uuid([0; 16]),
        AspectValue::String(InternedString::Symbol(Symbol(3))),
    ] {
        assert_eq!(value.owned_allocation_capacity_bytes(), 0);
    }
}

fn retained(value: &str, capacity: usize) -> String {
    let mut retained = String::with_capacity(capacity);
    retained.push_str(value);
    assert_eq!(retained.capacity(), capacity);
    retained
}

mod cost {
    use super::*;

    #[test]
    fn inline_bytes_clone_quote_counts_length_without_spare_capacity() {
        let length = 65536;
        let mut bytes = Vec::with_capacity(2 * length);
        bytes.resize(length, 0x51);
        let value = AspectValue::InlineBytes(crate::values::InlineBytes::try_new(bytes).unwrap());
        assert_eq!(value.owned_allocation_capacity_bytes(), 2 * length);
        assert_eq!(value.clone_allocation_bytes(), Some(length));
        assert_eq!(value.clone().owned_allocation_capacity_bytes(), length);
    }
}
