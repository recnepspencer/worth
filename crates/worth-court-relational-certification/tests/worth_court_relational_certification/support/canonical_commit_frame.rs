//! Independently reseal actual hostile v5 DATA; do not invoke producer commitments.
use sha2::{Digest, Sha256};

pub(crate) fn reseal(frame: &mut [u8]) {
    assert_eq!(&frame[..8], b"WRTHRCCA");
    assert_eq!(u16::from_le_bytes(frame[8..10].try_into().unwrap()), 5);
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.canonical.v5\0");
    digest.update(&frame[10..26]);
    for name in ["position", "canonical", "original_inputs"] {
        digest.update(Sha256::digest(crate::messagepack_value_spans::field(
            &frame[66..],
            name,
        )));
    }
    frame[26..58].copy_from_slice(&digest.finalize());
    let length = (frame.len() - 66) as u64;
    frame[58..66].copy_from_slice(&length.to_le_bytes());
}
