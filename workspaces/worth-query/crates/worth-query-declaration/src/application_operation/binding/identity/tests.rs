use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Serialize, Serializer};

use super::{canonical_identity, encoder};

fn identity<T: Serialize + ?Sized>(value: &T) -> [u8; 32] {
    canonical_identity("test", "scope", value)
        .expect("canonical encoding")
        .identity()
}

#[derive(Serialize)]
struct Keys {
    left: String,
    right: String,
}

#[derive(Serialize)]
struct KeysWithCount {
    left: String,
    right: String,
    count: u64,
}

#[test]
fn delimiter_text_inside_fields_does_not_collide() {
    let split_early = Keys {
        left: "a".into(),
        right: "b:c".into(),
    };
    let split_late = Keys {
        left: "a:b".into(),
        right: "c".into(),
    };
    assert_ne!(identity(&split_early), identity(&split_late));
}

#[test]
fn every_field_takes_part() {
    let with_count = |count| KeysWithCount {
        left: "a".into(),
        right: "b".into(),
        count,
    };
    assert_ne!(identity(&with_count(1)), identity(&with_count(2)));
}

#[test]
fn equal_values_share_one_identity() {
    let first = Keys {
        left: "a".into(),
        right: "b".into(),
    };
    let second = Keys {
        left: "a".into(),
        right: "b".into(),
    };
    assert_eq!(identity(&first), identity(&second));
}

#[test]
fn domain_and_scope_separate_equal_values() {
    let value = "same";
    let base = canonical_identity("domain", "scope", value)
        .unwrap()
        .identity();
    assert_ne!(
        base,
        canonical_identity("other", "scope", value)
            .unwrap()
            .identity()
    );
    assert_ne!(
        base,
        canonical_identity("domain", "other", value)
            .unwrap()
            .identity()
    );
    assert_ne!(
        canonical_identity("ab", "c", value).unwrap().identity(),
        canonical_identity("a", "bc", value).unwrap().identity()
    );
}

#[test]
fn map_order_does_not_change_identity() {
    let mut hashed = HashMap::new();
    let mut ordered = BTreeMap::new();
    for index in 0..64_u32 {
        hashed.insert(format!("key-{index}"), index);
        ordered.insert(format!("key-{index}"), index);
    }
    assert_eq!(identity(&hashed), identity(&ordered));
}

#[test]
fn floats_encode_exact_bits() {
    assert_ne!(identity(&0.0_f64), identity(&-0.0_f64));
    assert_ne!(identity(&0.1_f64), identity(&0.1_f32));
    assert_eq!(identity(&1.5_f64), identity(&1.5_f64));
}

#[test]
fn nesting_boundaries_are_encoded() {
    let flat: Vec<Vec<u8>> = vec![vec![1, 2], vec![3]];
    let shifted: Vec<Vec<u8>> = vec![vec![1], vec![2, 3]];
    assert_ne!(identity(&flat), identity(&shifted));
    assert_ne!(identity(&Some(Some(1_u8))), identity(&Some(1_u8)));
    assert_ne!(identity(&Option::<u8>::None), identity(&()));
}

#[derive(Serialize)]
enum Shape {
    Empty,
    Wrapped(u8),
    Pair(u8, u8),
    Named { value: u8 },
}

#[test]
fn enum_variants_are_distinct() {
    let identities = [
        identity(&Shape::Empty),
        identity(&Shape::Wrapped(1)),
        identity(&Shape::Pair(1, 0)),
        identity(&Shape::Named { value: 1 }),
    ];
    for (index, left) in identities.iter().enumerate() {
        for right in &identities[index + 1..] {
            assert_ne!(left, right);
        }
    }
}

struct Refuses;

impl Serialize for Refuses {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("refused"))
    }
}

#[test]
fn a_refusing_serializer_has_no_identity() {
    assert!(encoder::encode_into(&mut Vec::new(), &Refuses).is_err());
    assert!(canonical_identity("test", "scope", &vec![Refuses]).is_err());
}

fn encoded<T: Serialize + ?Sized>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    encoder::encode_into(&mut bytes, value).expect("canonical encoding");
    bytes
}

#[test]
fn small_integers_encode_compactly_and_by_value() {
    assert_eq!(encoded(&0_u8).len(), 2);
    assert_eq!(encoded(&127_u64).len(), 2);
    assert_eq!(encoded(&128_u64).len(), 3);
    assert_eq!(encoded(&-64_i64).len(), 2);
    assert_eq!(encoded(&u128::MAX).len(), 20);
    assert_eq!(encoded(&7_u8), encoded(&7_u128));
    assert_eq!(encoded(&-7_i8), encoded(&-7_i128));
}

#[test]
fn integers_are_injective_across_sign_and_width() {
    let mut seen = HashSet::new();
    let mut record = |bytes: Vec<u8>| assert!(seen.insert(bytes), "two integers share an encoding");
    for value in 0..70_000_u32 {
        record(encoded(&value));
    }
    for value in -70_000..0_i32 {
        record(encoded(&value));
    }
    for value in [u64::MAX, u64::from(u32::MAX) + 1, u64::MAX - 1] {
        record(encoded(&value));
    }
    for value in [
        i128::MIN,
        i128::MAX,
        i128::from(i64::MIN),
        i128::from(i64::MAX),
    ] {
        record(encoded(&value));
    }
    record(encoded(&u128::MAX));
    record(encoded(&(u128::MAX - 1)));
    // Signed zero is not unsigned zero: the sign tag keeps them apart.
    assert_ne!(encoded(&0_i32), encoded(&0_u32));
}

#[test]
fn integer_sequences_stay_delimited() {
    let lengths: [Vec<u16>; 4] = [vec![], vec![0], vec![0, 0], vec![128]];
    let mut seen = HashSet::new();
    for value in &lengths {
        assert!(seen.insert(encoded(value)));
    }
    assert_ne!(identity(&(1_u8, 2_u8)), identity(&(1_u16, 2_u16, 0_u8)));
    assert_ne!(
        identity(&vec![vec![1_u8], vec![]]),
        identity(&vec![vec![], vec![1_u8]])
    );
}

/// The encoding of a value as text, for values whose framing bytes are ASCII.
fn embedded<T: Serialize + ?Sized>(value: &T) -> String {
    String::from_utf8(encoded(value)).expect("framing bytes are ASCII")
}

fn all_distinct(identities: &[[u8; 32]]) {
    for (index, left) in identities.iter().enumerate() {
        for right in &identities[index + 1..] {
            assert_ne!(left, right);
        }
    }
}

#[test]
fn text_holding_the_encoders_own_framing_bytes_does_not_collide() {
    // Every tag, END and MORE byte and a short length are ASCII, so a string
    // can carry the exact encoding of another value.
    let pair = vec!["a", "b"];
    let smuggled_pair = vec![embedded(&pair)];
    assert_ne!(identity(&pair), identity(&smuggled_pair));
    let split_early = Keys {
        left: "a\u{0}".into(),
        right: "\u{1}b".into(),
    };
    let split_late = Keys {
        left: "a".into(),
        right: "\u{0}\u{1}b".into(),
    };
    assert_ne!(identity(&split_early), identity(&split_late));
    let tagged = Keys {
        left: "\u{7}\u{1}a".into(),
        right: String::new(),
    };
    let plain = Keys {
        left: String::new(),
        right: "\u{7}\u{1}a".into(),
    };
    assert_ne!(identity(&tagged), identity(&plain));
    assert_ne!(identity(&"a"), identity(&embedded(&"a")));
}

struct RawBytes<'a>(&'a [u8]);

impl Serialize for RawBytes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

#[test]
fn bytes_differ_from_a_sequence_of_u8() {
    let bytes = [1_u8, 2, 3];
    all_distinct(&[
        identity(&RawBytes(&bytes)),
        identity(&bytes.to_vec()),
        identity(&(1_u8, 2_u8, 3_u8)),
    ]);
    assert_ne!(identity(&RawBytes(&[])), identity(&Vec::<u8>::new()));
}

#[test]
fn a_char_differs_from_a_one_character_string() {
    assert_ne!(identity(&'a'), identity(&"a"));
    assert_ne!(identity(&'a'), identity(&"a".to_string()));
    assert_eq!(identity(&"a"), identity(&"a".to_string()));
}

#[test]
fn unit_none_and_the_empty_sequence_are_distinct() {
    all_distinct(&[
        identity(&()),
        identity(&Option::<u8>::None),
        identity(&Vec::<u8>::new()),
        identity(&BTreeMap::<u8, u8>::new()),
        identity(&Some(())),
    ]);
}

#[test]
fn a_derivation_reports_the_exact_work_that_produced_it() {
    // A string encodes as tag, one length byte and its text; the domain and
    // scope each add an eight-byte length and their text to the hashed bytes.
    let derived = canonical_identity("test", "scope", "abc").expect("canonical encoding");
    let work = derived.work();
    assert_eq!(work.derivations(), 1);
    assert_eq!(work.encoded_bytes(), 1 + 1 + 3);
    assert_eq!(work.sha256_input_bytes(), (8 + 4) + (8 + 5) + 5);
    assert_eq!(work.sha256_compression_blocks(), 1);
    assert_eq!(work.buffered_bytes(), 0);
    assert_eq!(derived.work(), work, "counts do not drift between reads");
}

#[test]
fn work_grows_with_the_value_and_adds_across_derivations() {
    let small = canonical_identity("test", "scope", &vec![1_u8; 4])
        .unwrap()
        .work();
    let large = canonical_identity("test", "scope", &vec![1_u8; 400])
        .unwrap()
        .work();
    assert!(large.encoded_bytes() > small.encoded_bytes());
    assert!(large.sha256_compression_blocks() > small.sha256_compression_blocks());
    let both = small.combine(large);
    assert_eq!(both.derivations(), 2);
    assert_eq!(
        both.encoded_bytes(),
        small.encoded_bytes() + large.encoded_bytes()
    );
    assert_eq!(
        both.sha256_compression_blocks(),
        small.sha256_compression_blocks() + large.sha256_compression_blocks()
    );
}

#[test]
fn map_entries_report_the_bytes_held_to_sort_them() {
    let mut flat = BTreeMap::new();
    flat.insert("k".to_string(), 1_u8);
    let flat = canonical_identity("test", "scope", &flat).unwrap().work();
    // One entry buffers its key (tag, length, text) and its value (tag, byte).
    assert_eq!(flat.buffered_bytes(), 3 + 2);
    let mut inner = BTreeMap::new();
    inner.insert("k".to_string(), 1_u8);
    let mut nested = BTreeMap::new();
    nested.insert("k".to_string(), inner);
    let nested = canonical_identity("test", "scope", &nested).unwrap().work();
    assert!(nested.buffered_bytes() > flat.buffered_bytes());
}
