//! Decimal selection borrows exact bytes; only native codec admission gives them validity.
use super::super::{canonical_identity, canonical_identity_with_capture, encoder};
use super::*;
use serde::Serialize;
fn capture(value: &impl Serialize) -> ApplicationCanonicalInput {
    let original = canonical_identity("decimal", "input", value).unwrap();
    let captured = canonical_identity_with_capture(
        "decimal",
        "input",
        value,
        encoder::HashingSink::new().capturing(true),
    )
    .unwrap();
    assert_eq!(original.identity(), captured.canonical.identity());
    captured.capture.unwrap()
}
#[test]
fn decimal_selection_preserves_required_and_optional_text_without_admitting_validity() {
    #[derive(Serialize)]
    struct Input<'a> {
        amount: &'a str,
        optional: Option<&'a str>,
        following: u64,
    }
    for optional in [None, Some("0"), Some("9007199254740993.125")] {
        let input = capture(&Input {
            amount: "9007199254740993.125",
            optional,
            following: 42,
        });
        let selected = input
            .scalar_member("amount", ScalarAspectType::Decimal)
            .unwrap();
        assert!(matches!(
            &selected,
            ApplicationCanonicalInputScalar::DecimalText("9007199254740993.125")
        ));
        assert_eq!(
            selected.owned_allocation_capacity_bytes(),
            "9007199254740993.125".len()
        );
        assert_eq!(
            selected.into_value(),
            AspectValue::Decimal(CanonicalDecimal::new("9007199254740993.125"))
        );
        assert_eq!(
            input
                .optional_scalar_member("optional", ScalarAspectType::Decimal)
                .unwrap()
                .map(ApplicationCanonicalInputScalar::into_value),
            optional.map(|v| AspectValue::Decimal(CanonicalDecimal::new(v)))
        );
        assert_eq!(
            input
                .scalar_member("following", ScalarAspectType::UInt64)
                .unwrap()
                .into_value(),
            AspectValue::UInt64(42)
        );
    }
    let unadmitted = capture(&Input {
        amount: "not-a-decimal",
        optional: None,
        following: 42,
    });
    assert!(matches!(
        unadmitted.scalar_member("amount", ScalarAspectType::Decimal),
        Ok(ApplicationCanonicalInputScalar::DecimalText(
            "not-a-decimal"
        ))
    ));
}
#[test]
fn decimal_selection_never_coerces_numbers_options_or_nominal_wrappers() {
    #[derive(Serialize)]
    struct Input<T> {
        amount: T,
    }
    for input in [
        capture(&Input { amount: 42_u64 }),
        capture(&Input { amount: 1.25_f64 }),
        capture(&Input {
            amount: Some("1.25"),
        }),
    ] {
        assert!(input
            .scalar_member("amount", ScalarAspectType::Decimal)
            .is_err());
    }
    #[derive(Serialize)]
    struct Nominal(String);
    assert!(capture(&Input {
        amount: Nominal("1.25".into())
    })
    .scalar_member("amount", ScalarAspectType::Decimal)
    .is_err());
    assert!(capture(&Input {
        amount: Some(Some("1.25"))
    })
    .optional_scalar_member("amount", ScalarAspectType::Decimal)
    .is_err());
}
