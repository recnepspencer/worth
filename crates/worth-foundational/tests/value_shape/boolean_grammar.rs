//! Actual array aliases, platform indices and named enum payloads share one wire grammar.
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_foundational::facade::{
    decode_canonical_value, decode_canonical_value_with_admission, encode_canonical_value,
    CanonicalValue, ValueCodecDenial as D, ValueDecodeBounds, WorthValue,
};

type Entity = [u64; 3];
type Nodes = [[usize; 3]; 2];
#[derive(Debug, PartialEq, WorthValue)]
enum Meaning {
    Empty,
    Name(String),
    Face {
        #[serde(rename = "source")]
        entity: Entity,
        nodes: Nodes,
        contributors: Vec<(usize, [u64; 6])>,
    },
}
const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(4096, 256, 64, 32);
const RAW: &str = r#"{"Face":{"source":[9,17,4],"nodes":[[1,2,3],[4,5,6]],"contributors":[[7,[11,12,13,14,15,16]]]}}"#;

#[test]
fn named_variant_and_nested_arrays_change_actual_wire_and_admitted_value() {
    let value = Meaning::Face {
        entity: [9, 17, 4],
        nodes: [[1, 2, 3], [4, 5, 6]],
        contributors: vec![(7, [11, 12, 13, 14, 15, 16])],
    };
    assert_eq!(encode_canonical_value(&value, BOUNDS).unwrap(), RAW);
    let mut calls = 0;
    let decoded = decode_canonical_value_with_admission::<Meaning>(RAW, BOUNDS, &mut |_, _| {
        calls += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(decoded, value);
    assert!(
        calls > 6,
        "the nested array/payload seeds use the original loan"
    );
    let shape = format!("{:?}", Meaning::value_shape());
    assert!(shape.contains("source"));
    assert!(shape.contains("Tuple"));
    for changed in [
        RAW.replace("[9,17,4]", "[9,17]"),
        RAW.replace("[9,17,4]", "[9,17,4,8]"),
        RAW.replace("\"source\"", "\"unknown\""),
        RAW.replace("\"Face\"", "\"Missing\""),
    ] {
        assert!(decode_canonical_value::<Meaning>(&changed, BOUNDS).is_err());
    }
    assert_eq!(
        decode_canonical_value_with_admission::<Meaning>(RAW, BOUNDS, &mut |_, _| Err(
            D::Admission
        )),
        Err(D::Admission)
    );
}

#[test]
fn platform_index_is_checked_against_the_u64_wire() {
    let raw = u64::MAX.to_string();
    match usize::try_from(u64::MAX) {
        Ok(value) => {
            assert_eq!(
                decode_canonical_value::<usize>(&raw, BOUNDS).unwrap(),
                value
            );
            assert_eq!(encode_canonical_value(&value, BOUNDS).unwrap(), raw);
        }
        Err(_) => assert_eq!(
            decode_canonical_value::<usize>(&raw, BOUNDS),
            Err(D::Malformed)
        ),
    }
    for raw in ["-1", "18446744073709551616", "1.5"] {
        assert_eq!(
            decode_canonical_value::<usize>(raw, BOUNDS),
            Err(D::Malformed)
        );
    }
    assert_eq!(usize::value_shape(), u64::value_shape());
}

static DROPPED: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug, WorthValue)]
struct Element(u64);
impl Drop for Element {
    fn drop(&mut self) {
        DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn partial_fixed_array_refusal_drops_only_its_original_decoded_elements() {
    for (raw, expected) in [("[1,2]", 2), ("[1,2,3,4]", 3)] {
        DROPPED.store(0, Ordering::SeqCst);
        assert!(decode_canonical_value::<[Element; 3]>(raw, BOUNDS).is_err());
        assert_eq!(DROPPED.load(Ordering::SeqCst), expected);
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
struct DomainValue(u64);
impl DomainValue {
    fn try_new(value: u64) -> Result<Self, D> {
        if value == 0 {
            Err(D::NonCanonical)
        } else {
            Ok(Self(value))
        }
    }
}
struct DomainDelegate;
impl worth_foundational::facade::ValueSeedDelegate for DomainDelegate {
    type Value = DomainValue;
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: worth_foundational::facade::ValueShapeBody =
        <u64 as CanonicalValue>::SHAPE_BODY;
    fn validate(value: &DomainValue, bounds: ValueDecodeBounds) -> Result<(), D> {
        DomainValue::try_new(value.0)?;
        value.0.validate_bounds(bounds)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        mut admission: Option<&mut worth_foundational::facade::ValueDecodeAdmission<'_>>,
    ) -> Result<DomainValue, Decoder::Error> {
        let value = serde::de::DeserializeSeed::deserialize(
            worth_foundational::facade::CanonicalValueSeed::<u64>::with_optional_admission(
                bounds,
                admission.as_deref_mut(),
            ),
            decoder,
        )?;
        DomainValue::try_new(value)
            .map_err(|cause| worth_foundational::facade::value_decode_error(cause, admission))
    }
}
#[derive(Debug, PartialEq, WorthValue)]
struct Delegated {
    #[worth_value(delegate = DomainDelegate)]
    chosen: DomainValue,
}
#[test]
fn explicit_domain_field_delegate_preserves_wire_constructor_and_original_refusal() {
    let value = Delegated {
        chosen: DomainValue::try_new(17).unwrap(),
    };
    let raw = r#"{"chosen":17}"#;
    assert_eq!(encode_canonical_value(&value, BOUNDS).unwrap(), raw);
    assert_eq!(serde_json::to_string(&value.chosen).unwrap(), "17");
    assert_eq!(
        decode_canonical_value::<Delegated>(raw, BOUNDS).unwrap(),
        value
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Delegated>(raw, BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        value
    );
    assert_eq!(
        decode_canonical_value::<Delegated>(r#"{"chosen":0}"#, BOUNDS),
        Err(D::NonCanonical)
    );
    assert_eq!(
        encode_canonical_value(
            &Delegated {
                chosen: DomainValue(0)
            },
            BOUNDS
        ),
        Err(D::NonCanonical)
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Delegated>(raw, BOUNDS, &mut |_, _| Err(
            D::Admission
        )),
        Err(D::Admission)
    );
}
