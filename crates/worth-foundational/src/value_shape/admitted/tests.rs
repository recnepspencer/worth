use super::*;
use crate::value_shape::{CanonicalValue, ValueShapeBody, ValueShapeEdge};
use crate::values::InternedString;
use std::cell::Cell;

thread_local! {
    static SERIALIZATIONS: Cell<usize> = const { Cell::new(0) };
    static VALIDATIONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Debug, PartialEq, Eq)]
struct Text(String);
impl Serialize for Text {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SERIALIZATIONS.with(|count| count.set(count.get() + 1));
        self.0.serialize(serializer)
    }
}
impl CanonicalValue for Text {
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody =
        ValueShapeBody::Transparent(ValueShapeEdge::value::<String>());
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        VALIDATIONS.with(|count| count.set(count.get() + 1));
        self.0.validate_bounds(bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        String::deserialize_bounded(decoder, bounds).map(Self)
    }
    fn deserialize_reporting<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        reporter: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        String::deserialize_reporting(decoder, bounds, reporter).map(Self)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        String::deserialize_admitted(decoder, bounds, admission).map(Self)
    }
}

fn counts() -> (usize, usize) {
    (SERIALIZATIONS.with(Cell::get), VALIDATIONS.with(Cell::get))
}

#[test]
fn stored_reads_carry_the_codec_proof_and_restore_rechecks_untrusted_bytes() {
    let bounds = ValueDecodeBounds::unrestricted();
    let value = Text("alpha".to_owned());
    let text = encode_stored_canonical_value(&value, bounds).unwrap();
    let before = counts();
    for _ in 0..16 {
        assert_eq!(
            decode_stored_canonical_value::<Text>(&text, bounds, None).unwrap(),
            value
        );
    }
    assert_eq!(
        counts(),
        before,
        "reads must consume the seal without revalidating or serializing"
    );
    let wire = serde_json::to_string(&text).unwrap();
    assert_eq!(wire, r#"{"Raw":"\"alpha\""}"#);
    let restored: InternedString = serde_json::from_str(&wire).unwrap();
    assert_eq!(restored, text);
    let before = counts();
    assert_eq!(
        decode_stored_canonical_value::<Text>(&restored, bounds, None).unwrap(),
        value
    );
    assert!(counts().0 > before.0 && counts().1 > before.1);
    let checked = counts();
    assert_eq!(
        decode_stored_canonical_value::<Text>(&restored, bounds, None).unwrap(),
        value
    );
    assert_eq!(counts(), checked);
    assert_eq!(
        decode_stored_canonical_value::<Text>(&text, bounds.with_string_limit(2), None),
        Err(D::StringBytes)
    );
    assert_eq!(
        decode_stored_canonical_value::<Text>(&text, bounds, Some(&mut |_, _| Err(D::Admission))),
        Err(D::Admission)
    );
}

#[test]
fn corrupt_or_noncanonical_text_never_acquires_a_seal() {
    for raw in [r#""\u0061""#, r#" "alpha""#, r#""alpha" trailing"#] {
        let text = InternedString::Stored(StoredValueText::new(raw.to_owned()));
        for _ in 0..2 {
            assert!(decode_stored_canonical_value::<Text>(
                &text,
                ValueDecodeBounds::unrestricted(),
                None
            )
            .is_err());
        }
    }
    let text =
        encode_stored_canonical_value(&Text("alpha".to_owned()), ValueDecodeBounds::unrestricted())
            .unwrap();
    assert!(
        decode_stored_canonical_value::<u64>(&text, ValueDecodeBounds::unrestricted(), None)
            .is_err()
    );
}

#[test]
fn a_fixed_stored_read_stays_flat_with_bystanders_and_prior_reads() {
    fn read(model: usize, history: usize) -> usize {
        let bounds = ValueDecodeBounds::unrestricted();
        let text = encode_stored_canonical_value(&Text("fixed".to_owned()), bounds).unwrap();
        let bystanders = (0..model)
            .map(|i| {
                encode_stored_canonical_value(&Text(format!("bystander-{i}")), bounds).unwrap()
            })
            .collect::<Vec<_>>();
        for _ in 0..history {
            decode_stored_canonical_value::<Text>(&text, bounds, None).unwrap();
        }
        // Warm the independent static grammar admission proof as installation does.
        crate::value_shape::ValueShapeDefinition::of::<Text>()
            .inspect_admitted(&mut |_, _| Ok(()))
            .unwrap();
        let mut work = 0;
        decode_stored_canonical_value::<Text>(
            &text,
            bounds,
            Some(&mut |charge, _| {
                work += charge;
                Ok(())
            }),
        )
        .unwrap();
        assert_eq!(bystanders.len(), model);
        work
    }
    let baseline = read(128, 128);
    for actual in [read(256, 128), read(128, 256)] {
        assert!((actual as f64) / (baseline as f64) < 1.5);
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct DifferentText(String);
impl CanonicalValue for DifferentText {
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = Text::SHAPE_BODY;
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), D> {
        if self.0 == "beta" {
            Ok(())
        } else {
            Err(D::NonCanonical)
        }
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        String::deserialize_bounded(decoder, bounds).map(Self)
    }
}

#[test]
fn identical_wire_grammars_do_not_share_codec_or_policy_proof() {
    let bounds = ValueDecodeBounds::unrestricted();
    let text = encode_stored_canonical_value(&Text("alpha".to_owned()), bounds).unwrap();
    assert_eq!(
        decode_stored_canonical_value::<DifferentText>(&text, bounds, None),
        Err(D::NonCanonical)
    );
    let before = counts();
    decode_stored_canonical_value::<Text>(&text, bounds.with_string_limit(10), None).unwrap();
    assert!(
        counts().0 > before.0 && counts().1 > before.1,
        "different bounds require their own proof"
    );
}

#[test]
fn a_standalone_shape_proof_does_not_admit_an_unsupported_seed() {
    let definition = crate::value_shape::ValueShapeDefinition::of::<DifferentText>();
    definition.inspect().unwrap();
    assert_eq!(
        definition.inspect_admitted(&mut |_, _| Ok(())),
        Err(D::UnsupportedAdmission)
    );
}

#[test]
fn immutable_text_copies_carry_proof_without_sharing_allocation_custody() {
    let bounds = ValueDecodeBounds::unrestricted();
    let text = encode_stored_canonical_value(&Text("alpha".to_owned()), bounds).unwrap();
    let copy = text.clone();
    assert_eq!(text, copy);
    assert_ne!(
        text.raw_text().unwrap().as_ptr(),
        copy.raw_text().unwrap().as_ptr()
    );
    let before = counts();
    decode_stored_canonical_value::<Text>(&copy, bounds, None).unwrap();
    assert_eq!(
        counts(),
        before,
        "a byte-identical immutable copy carries the seal"
    );
}

#[test]
fn checking_a_restored_copy_seals_the_same_immutable_source_bytes() {
    let bounds = ValueDecodeBounds::unrestricted();
    let text: InternedString = serde_json::from_str(r#"{"Raw":"\"alpha\""}"#).unwrap();
    let copy = text.clone();
    decode_stored_canonical_value::<Text>(&copy, bounds, None).unwrap();
    let checked = counts();
    decode_stored_canonical_value::<Text>(&text, bounds, None).unwrap();
    assert_eq!(
        counts(),
        checked,
        "read copies carry proof back to the immutable stored artifact"
    );
}
