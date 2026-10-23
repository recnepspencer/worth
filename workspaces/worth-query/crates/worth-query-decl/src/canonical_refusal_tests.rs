//! Canonical application values require no lower platform dependency.
use crate::facade::application_schema::*;

#[derive(Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "crate::facade::application_schema")]
struct Refusal {
    code: u64,
    paths: Vec<(u64, String)>,
    reason: Reason,
}

#[derive(Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "crate::facade::application_schema")]
enum Reason {
    Conflict { names: [String; 2] },
    Empty,
}

crate::facade::worth_query_value_binding!(RefusalBinding for Refusal {
    identity: "tests.facade-only-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 128, 16, 16),
});

#[test]
fn canonical_refusal_uses_only_the_declared_audience_facade() {
    let value = Refusal {
        code: 17,
        paths: vec![(2, "edge".into())],
        reason: Reason::Conflict {
            names: ["a".into(), "b".into()],
        },
    };
    let encoded = RefusalBinding::encode(&value).unwrap();
    assert_eq!(RefusalBinding::decode(&encoded).unwrap(), value);
    assert!(RefusalBinding::canonical_contract()
        .unwrap()
        .supports_admission());
}

// A foreign domain value has no platform traits and does not need Clone.
#[derive(Debug, Eq, PartialEq, serde::Serialize)]
struct DomainRefusal {
    reason: String,
}
#[derive(WorthValue)]
#[worth_value(crate = "crate::facade::application_schema")]
struct RefusalGrammar {
    reason: String,
}
struct DomainDelegate;
impl ValueSeedDelegate for DomainDelegate {
    type Value = DomainRefusal;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = RefusalGrammar::SHAPE_BODY;
    fn validate(value: &DomainRefusal, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        value.reason.validate_bounds(bounds)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<DomainRefusal, Decoder::Error> {
        use serde::de::DeserializeSeed;
        let grammar =
            CanonicalValueSeed::<RefusalGrammar>::with_optional_admission(bounds, admission)
                .deserialize(decoder)?;
        Ok(DomainRefusal {
            reason: grammar.reason,
        })
    }
}

#[test]
fn foreign_refusal_uses_the_existing_canonical_codec_without_a_value_copy() {
    let bounds = ValueDecodeBounds::new(128, 64, 8, 8);
    let value = DomainRefusal {
        reason: "coincident".into(),
    };
    let bytes = encode_delegated_canonical_value::<DomainDelegate>(&value, bounds).unwrap();
    assert_eq!(bytes, r#"{"reason":"coincident"}"#);
    assert_eq!(
        bytes,
        encode_canonical_value(
            &RefusalGrammar {
                reason: "coincident".into()
            },
            bounds
        )
        .unwrap()
    );
    assert_eq!(
        decode_delegated_canonical_value::<DomainDelegate>(&bytes, bounds).unwrap(),
        value
    );
    assert_eq!(
        ApplicationCanonicalValueContract::delegated::<DomainDelegate>(bounds).shape(),
        ApplicationCanonicalValueContract::of::<RefusalGrammar>(bounds).shape()
    );
    assert!(decode_delegated_canonical_value::<DomainDelegate>(
        r#"{ "reason":"coincident"}"#,
        bounds
    )
    .is_err());
}

#[test]
fn foreign_refusal_decode_keeps_the_callers_admission_denial() {
    let mut consulted = false;
    let result = decode_delegated_canonical_value_with_admission::<DomainDelegate>(
        r#"{"reason":"coincident"}"#,
        ValueDecodeBounds::new(128, 64, 8, 8),
        &mut |_, _| {
            consulted = true;
            Err(ValueCodecDenial::Allocation)
        },
    );
    assert!(consulted);
    assert_eq!(result, Err(ValueCodecDenial::Allocation));
}
