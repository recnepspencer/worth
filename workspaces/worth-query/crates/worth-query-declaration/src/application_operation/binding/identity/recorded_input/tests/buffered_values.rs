//! Serde buffering must keep the existing signed and unsigned value families.
use super::{bounds, ApplicationEncodedInput, CanonicalDigestId};
use crate::facade::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Signed {
    Number(i64),
    Text(String),
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Unsigned {
    Number(u64),
    Text(String),
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Tagged {
    Values { signed: i64, unsigned: u64 },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Buffered {
    signed: Signed,
    unsigned: Unsigned,
    tagged: Tagged,
}
struct Binding;
impl ApplicationStructuredValueBinding for Binding {
    type Value = Buffered;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.buffered.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

#[test]
fn recorded_input_roundtrips_untagged_and_internally_tagged_numeric_buffers() {
    let input = Buffered {
        signed: Signed::Number(-7),
        unsigned: Unsigned::Number(u64::MAX),
        tagged: Tagged::Values {
            signed: i64::MIN,
            unsigned: 7,
        },
    };
    let encoded = ApplicationEncodedInput::<Binding>::encode(input).unwrap();
    let captured = encoded.capture_for_retention().unwrap();
    let restored = ApplicationEncodedInput::<Binding>::decode_retained(
        captured.recorded_bytes(),
        CanonicalDigestId::new(*encoded.identity()),
        bounds(),
    )
    .unwrap();
    assert_eq!(restored.input(), encoded.input());
    assert_eq!(restored.identity(), encoded.identity());
}
