//! Existing bounded value seeds can reconstruct inputs without DeserializeOwned.
use super::*;
use serde::de::DeserializeSeed;
use worth_foundational::facade::{CanonicalValueSeed, WorthValue};
#[derive(Clone, Debug, PartialEq, WorthValue)]
struct SeededInput {
    text: String,
    count: u64,
    state: State,
    tagged: TaggedState,
}
#[derive(Clone, Debug, PartialEq, WorthValue)]
enum State {
    Empty,
    Counter(u64),
    Named { flags: Vec<bool> },
}
#[derive(Clone, Debug, PartialEq, WorthValue)]
#[serde(tag = "posture")]
enum TaggedState {
    Paused,
    Ready { value: u64 },
}
struct SeededBinding;
impl crate::application_schema::ApplicationStructuredValueBinding for SeededBinding {
    type Value = SeededInput;
    const IDENTITY_NAME: &'static str = "recorded-input.seeded.v1";
    fn validate(
        _: &Self::Value,
    ) -> Result<(), crate::application_schema::ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[test]
fn recorded_input_reuses_the_existing_bounded_value_seed_without_a_serde_value_bound() {
    for state in [
        State::Empty,
        State::Counter(9),
        State::Named {
            flags: vec![true, false],
        },
    ] {
        for tagged in [TaggedState::Paused, TaggedState::Ready { value: 11 }] {
            let encoded = ApplicationEncodedInput::<SeededBinding>::encode(SeededInput {
                text: "source".into(),
                count: 7,
                state: state.clone(),
                tagged,
            })
            .unwrap();
            let capture = encoded.capture_for_retention().unwrap();
            let restored = ApplicationEncodedInput::<SeededBinding>::decode_retained_with_seed(
                capture.recorded_bytes(),
                CanonicalDigestId::new(*encoded.identity()),
                bounds(),
                CanonicalValueSeed::<SeededInput>::new(bounds()),
            )
            .unwrap();
            assert_eq!(restored.input(), encoded.input());
            assert_eq!(restored.identity(), encoded.identity());
        }
    }
}
struct ChangedSeed;
impl<'de> DeserializeSeed<'de> for ChangedSeed {
    type Value = SeededInput;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        let mut value = CanonicalValueSeed::<SeededInput>::new(bounds()).deserialize(decoder)?;
        value.count += 1;
        Ok(value)
    }
}
#[test]
fn recorded_input_refuses_a_seed_that_changes_the_original_value_identity() {
    let encoded = ApplicationEncodedInput::<SeededBinding>::encode(SeededInput {
        text: "source".into(),
        count: 7,
        state: State::Empty,
        tagged: TaggedState::Paused,
    })
    .unwrap();
    let capture = encoded.capture_for_retention().unwrap();
    assert!(matches!(
        ApplicationEncodedInput::<SeededBinding>::decode_retained_with_seed(
            capture.recorded_bytes(),
            CanonicalDigestId::new(*encoded.identity()),
            bounds(),
            ChangedSeed
        ),
        Err(Denial::IdentityMismatch)
    ));
}
