//! Pure codec observations, with no runtime owner or authority construction.
use super::BinaryFixedArrays;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Digests {
    direct: [u8; 32],
    nested: Option<([u8; 16], [u8; 32])>,
    numbers: [u64; 3],
}

#[test]
fn fixed_digest_encoding_has_value_independent_width_and_round_trips() {
    let encode = |byte| {
        let value = Digests {
            direct: [byte; 32],
            nested: Some(([byte; 16], [byte; 32])),
            numbers: [0, 128, 256],
        };
        let bytes = rmp_serde::to_vec_named(&BinaryFixedArrays(&value)).unwrap();
        assert_eq!(rmp_serde::from_slice::<Digests>(&bytes).unwrap(), value);
        assert_eq!(
            bytes
                .windows(2)
                .filter(|window| *window == [0xc4, 32])
                .count(),
            2
        );
        assert_eq!(
            bytes
                .windows(2)
                .filter(|window| *window == [0xc4, 16])
                .count(),
            1
        );
        bytes
    };
    let low = encode(0);
    let high = encode(255);
    assert_eq!(low.len(), high.len());
    assert!(!low.windows(3).any(|window| window == [0xdc, 0, 32]));
    assert!(!high.windows(3).any(|window| window == [0xdc, 0, 32]));
}

#[test]
fn fixed_byte_encoding_preserves_nonbyte_tuples_and_variable_byte_sequences() {
    // These representations must not depend on whether integers happen to fit u8.
    for numbers in [[0_u64; 32], [255_u64; 32], [256_u64; 32]] {
        let ordinary = rmp_serde::to_vec_named(&numbers).unwrap();
        let projected = rmp_serde::to_vec_named(&BinaryFixedArrays(&numbers)).unwrap();
        assert_eq!(projected, ordinary);
    }
    let mixed = (3_u8, 4_u16, [255_u8; 32]);
    let projected = rmp_serde::to_vec_named(&BinaryFixedArrays(&mixed)).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<(u8, u16, [u8; 32])>(&projected).unwrap(),
        mixed
    );
    assert_eq!(projected[0], 0x93);
    let variable = vec![255_u8; 32];
    assert_eq!(
        rmp_serde::to_vec_named(&BinaryFixedArrays(&variable)).unwrap(),
        rmp_serde::to_vec_named(&variable).unwrap(),
    );
}

#[test]
fn fixed_byte_probe_obeys_the_underlying_serializer_mode() {
    struct ModeSensitive;
    impl Serialize for ModeSensitive {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            if serializer.is_human_readable() {
                serializer.serialize_u8(7)
            } else {
                serializer.serialize_str("binary-mode")
            }
        }
    }
    let value = (ModeSensitive,);
    assert_eq!(
        rmp_serde::to_vec_named(&BinaryFixedArrays(&value)).unwrap(),
        rmp_serde::to_vec_named(&value).unwrap(),
    );
}

#[test]
fn fixed_byte_probe_does_not_format_nonbyte_tuple_values() {
    struct Formatted<'a>(&'a std::cell::Cell<usize>);
    impl std::fmt::Display for Formatted<'_> {
        fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.set(self.0.get() + 1);
            output.write_str("display-value")
        }
    }
    impl Serialize for Formatted<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }
    let calls = std::cell::Cell::new(0);
    let value = (Formatted(&calls),);
    let ordinary = rmp_serde::to_vec_named(&value).unwrap();
    assert_eq!(calls.replace(0), 1);
    assert_eq!(
        rmp_serde::to_vec_named(&BinaryFixedArrays(&value)).unwrap(),
        ordinary
    );
    assert_eq!(
        calls.get(),
        1,
        "the rejected probe must not render a temporary string"
    );
}
