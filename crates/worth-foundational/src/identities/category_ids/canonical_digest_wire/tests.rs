use super::CanonicalDigestId;

#[test]
fn binary_digest_width_is_independent_of_every_byte_value() {
    for byte in u8::MIN..=u8::MAX {
        let digest = CanonicalDigestId::new([byte; 32]);
        let encoded = rmp_serde::to_vec(&digest).unwrap();
        assert_eq!(encoded.len(), 34);
        assert_eq!(&encoded[..2], &[0xc4, 32]);
        assert_eq!(
            rmp_serde::from_slice::<CanonicalDigestId>(&encoded).unwrap(),
            digest
        );
    }
}

#[test]
fn binary_digest_admission_refuses_the_retired_integer_array() {
    for byte in [0_u8, 127, 128, 255] {
        let retired = rmp_serde::to_vec(&[byte; 32]).unwrap();
        assert!(rmp_serde::from_slice::<CanonicalDigestId>(&retired).is_err());
    }
}

#[test]
fn binary_digest_admission_requires_exactly_thirty_two_bytes() {
    for length in [0, 31, 33] {
        let mut encoded = vec![0xc4, length];
        encoded.resize(usize::from(length) + 2, 17);
        assert!(rmp_serde::from_slice::<CanonicalDigestId>(&encoded).is_err());
    }
}

#[test]
fn human_readable_digest_has_fixed_width_and_refuses_the_retired_array() {
    let digest = CanonicalDigestId::new([173; 32]);
    let encoded = serde_json::to_string(&digest).unwrap();
    assert_eq!(encoded.len(), 66);
    assert_eq!(
        serde_json::from_str::<CanonicalDigestId>(&encoded).unwrap(),
        digest
    );
    assert_eq!(
        serde_json::from_str::<String>(&encoded).unwrap(),
        "ad".repeat(32)
    );
    let retired = serde_json::to_string(&[173_u8; 32]).unwrap();
    assert!(serde_json::from_str::<CanonicalDigestId>(&retired).is_err());
    assert!(serde_json::from_str::<CanonicalDigestId>("\"ad\"").is_err());
}
