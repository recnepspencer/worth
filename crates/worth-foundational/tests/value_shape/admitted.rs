//! Representation bounds do not replace cumulative borrowed request authority.
use super::{packet, Packet, BOUNDS, RAW};
use worth_foundational::facade::{
    decode_canonical_value_with_admission, CanonicalValue, ValueCodecDenial as D,
    ValueDecodeAdmission, ValueDecodeBounds, ValueShapeBody, WorthValue,
};

#[test]
fn one_loan_covers_nested_values_and_parser_escape_scratch_before_visits() {
    let mut calls = Vec::new();
    let value = decode_canonical_value_with_admission::<Packet>(RAW, BOUNDS, &mut |work, bytes| {
        calls.push((work, bytes));
        Ok(())
    })
    .unwrap();
    assert_eq!(value, packet());
    // Independent arithmetic for parser scratch, diagnostic String growth and
    // boxed-str conversion, depth-bounded JSON error boxes, and IO error payload.
    let word = std::mem::size_of::<usize>();
    let expected = 4 * (RAW.len() + 4)
        + 16
        + 5 * (6 * RAW.len() + 128)
        + (BOUNDS.maximum_depth() + 3)
            * (std::mem::size_of::<std::io::Error>().max(2 * word) + 3 * word)
        + 4 * word
        + std::mem::size_of::<D>();
    assert_eq!(calls[0], (RAW.len(), expected));
    // More than the initial parser charge: owned keys/strings and Vec storage.
    assert!(calls.iter().skip(1).any(|&(_, bytes)| bytes > 0));

    let escaped = r#""line\nδ""#;
    let mut grants = 0;
    assert_eq!(
        decode_canonical_value_with_admission::<String>(escaped, BOUNDS, &mut |_, bytes| {
            grants += 1;
            // A wire-length-only grant cannot fund parser scratch and reallocations.
            if bytes > escaped.len() {
                Err(D::Admission)
            } else {
                Ok(())
            }
        }),
        Err(D::Admission)
    );
    assert_eq!(grants, 1);
    assert_eq!(
        decode_canonical_value_with_admission::<String>(escaped, BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        "line\nδ"
    );
}

#[derive(Debug, PartialEq, serde::Serialize)]
struct Swallowing(bool);
impl CanonicalValue for Swallowing {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { bool::SHAPE_BODY };
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> {
        Ok(())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        bool::deserialize_bounded(decoder, bounds).map(Self)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        let _ignored = admission(1, 1000);
        Self::deserialize_bounded(decoder, bounds)
    }
}

#[test]
fn swallowed_seed_refusal_wins_and_new_funded_loan_succeeds() {
    let mut requests = 0;
    let mut refused = false;
    assert_eq!(
        decode_canonical_value_with_admission::<Swallowing>("true", BOUNDS, &mut |_, bytes| {
            requests += 1;
            assert!(
                !refused,
                "authority must not be re-entered after its first refusal"
            );
            if bytes == 1000 {
                refused = true;
                Err(D::Admission)
            } else {
                Ok(())
            }
        }),
        Err(D::Admission)
    );
    assert!(refused);
    assert!(requests > 1);
    assert_eq!(
        decode_canonical_value_with_admission::<Swallowing>("true", BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        Swallowing(true)
    );
}

#[derive(Debug, WorthValue)]
struct LongGrammar {
    #[serde(
        rename = "a_static_declared_field_name_whose_size_is_unrelated_to_the_two_byte_empty_object_and_must_never_be_rendered_as_an_unfunded_diagnostic_in_a_native_request"
    )]
    value: bool,
}

#[test]
fn generated_grammar_diagnostics_never_render_static_names_or_lists() {
    for raw in ["{}", r#"{"unknown":true}"#] {
        let mut decoder = serde_json::Deserializer::from_str(raw);
        let error = LongGrammar::deserialize_bounded(&mut decoder, BOUNDS).unwrap_err();
        assert!(error.to_string().starts_with(&D::Malformed.to_string()));
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
struct Unadmitted(bool);
impl CanonicalValue for Unadmitted {
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { bool::SHAPE_BODY };
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> {
        Ok(())
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        bool::deserialize_bounded(decoder, bounds).map(Self)
    }
}

#[test]
fn opaque_owner_without_admitted_seed_has_no_deserialize_fallback() {
    assert_eq!(
        decode_canonical_value_with_admission::<Unadmitted>("true", BOUNDS, &mut |_, _| panic!(
            "unsupported codec must refuse before parser admission"
        )),
        Err(D::UnsupportedAdmission)
    );
}
