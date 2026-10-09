//! Focused binding dispatch tests, without graph/currentness claims.
use super::decode_admitted;
use worth_foundational::facade::{AspectValue, InternedString};

worth_query_declaration::worth_query_value_binding!(TestCarrierBinding for String {
    identity: "worth.query.test.predecode-carrier.v1", scalar: String,
    validate: validate, encode: encode, decode: decode,
});

fn validate(value: &String) -> Result<(), &'static str> {
    (value == "accepted")
        .then_some(())
        .ok_or("invalid decoded domain value")
}
fn encode(value: &str) -> InternedString {
    value.to_owned().into()
}
fn decode(value: InternedString) -> Option<String> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    assert_ne!(value, "decoder-must-not-run");
    (value != "malformed").then_some(value)
}

#[test]
fn predecode_denial_prevents_actual_declared_binding_dispatch() {
    let raw = AspectValue::String("decoder-must-not-run".into());
    let result =
        decode_admitted::<TestCarrierBinding, _>(&raw, |_, _| Err("owner-denied"), &|| Ok(()));
    assert_eq!(result.unwrap(), Err("owner-denied"));
}

#[test]
fn accepted_admission_preserves_decode_rejection_and_mandatory_macro_validation() {
    for raw in [
        AspectValue::String("malformed".into()),
        AspectValue::String("decoder-succeeds-validator-denies".into()),
        AspectValue::UInt64(5),
    ] {
        assert_eq!(
            decode_admitted::<TestCarrierBinding, ()>(&raw, |_, _| Ok(()), &|| Ok(()))
                .unwrap()
                .unwrap(),
            None
        );
    }
    let raw = AspectValue::String("accepted".into());
    assert_eq!(
        decode_admitted::<TestCarrierBinding, ()>(&raw, |_, _| Ok(()), &|| Ok(()))
            .unwrap()
            .unwrap(),
        Some("accepted".to_owned())
    );
}
