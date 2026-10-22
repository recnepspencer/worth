//! Canonical grammar admission and semantic domain admission remain independent.
mod document;
use super::*;
use worth_foundational::facade::{AspectValue, WorthValue};

#[derive(Debug, Eq, PartialEq, WorthValue)]
struct Span {
    length: i64,
    label: String,
}
fn positive(value: &Span) -> Result<(), &'static str> {
    (value.length > 0)
        .then_some(())
        .ok_or("span length must be positive")
}
crate::worth_query_value_binding!(SpanBinding for Span {
    identity: "tests.span.v1", scalar: String, canonical, validate: positive,
});
crate::worth_query_value_binding!(PolicyBinding for String {
    identity: "tests.policy.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024,3,8,4),
});

#[test]
fn generated_canonical_binding_validates_both_directions_and_keeps_exact_causes() {
    let value = Span {
        length: 17,
        label: "δ".into(),
    };
    let encoded = SpanBinding::encode(&value).unwrap();
    assert_eq!(
        encoded,
        AspectValue::String(r#"{"length":17,"label":"δ"}"#.into())
    );
    assert_eq!(
        SpanBinding::decode_admitted(&encoded, &mut |_, _| Ok(())).unwrap(),
        value
    );
    let invalid = Span {
        length: 0,
        label: "invalid".into(),
    };
    assert!(matches!(
        SpanBinding::encode(&invalid),
        Err(ApplicationValueEncodeDenial::Validation(_))
    ));
    let invalid = AspectValue::String(r#"{"length":0,"label":"invalid"}"#.into());
    assert!(matches!(
        SpanBinding::decode(&invalid),
        Err(ApplicationValueDecodeDenial::Validation(_))
    ));
    assert!(matches!(
        SpanBinding::decode_admitted(&invalid, &mut |_, _| Ok(())),
        Err(ApplicationValueDecodeDenial::Validation(_))
    ));
    assert_eq!(
        PolicyBinding::decode(&AspectValue::String(r#""long""#.into())),
        Err(ApplicationValueDecodeDenial::Canonical {
            binding_identity: PolicyBinding::IDENTITY,
            cause: ValueCodecDenial::StringBytes
        })
    );
    assert_eq!(
        PolicyBinding::encode(&"long".into()),
        Err(ApplicationValueEncodeDenial::Canonical {
            binding_identity: PolicyBinding::IDENTITY,
            cause: ValueCodecDenial::StringBytes
        })
    );
    assert_eq!(
        SpanBinding::decode(&AspectValue::String("{".into())),
        Err(ApplicationValueDecodeDenial::Canonical {
            binding_identity: SpanBinding::IDENTITY,
            cause: ValueCodecDenial::Malformed
        })
    );
}
