//! Buffered content must reconstruct all existing external enum payload shapes.
use super::{bounds, ApplicationEncodedInput, CanonicalDigestId};
use crate::facade::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Mode {
    Active,
    Count(u64),
    Pair(i64, String),
    Fields { signed: i64, unsigned: u64 },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Tagged {
    Set { mode: Mode },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Mode(Mode),
    Other(Vec<u64>),
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Buffered {
    tagged: Tagged,
    untagged: Untagged,
}
struct Binding;
impl ApplicationStructuredValueBinding for Binding {
    type Value = Buffered;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.buffered-enums.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[test]
fn recorded_input_roundtrips_external_enum_payloads_in_tagged_and_untagged_buffers() {
    for mode in [
        Mode::Active,
        Mode::Count(u64::MAX),
        Mode::Pair(-7, "pair".to_owned()),
        Mode::Fields {
            signed: i64::MIN,
            unsigned: 7,
        },
    ] {
        let encoded = ApplicationEncodedInput::<Binding>::encode(Buffered {
            tagged: Tagged::Set { mode: mode.clone() },
            untagged: Untagged::Mode(mode),
        })
        .unwrap();
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
}
