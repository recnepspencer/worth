//! Deliberately resealed hostile frames, independently composing the v8 commitment.
use sha2::{Digest, Sha256};

pub(crate) fn reseal_canonical_commit_frame(bytes: &mut [u8]) {
    assert_eq!(&bytes[..8], b"WRTHRCCA");
    assert_eq!(u16::from_le_bytes(bytes[8..10].try_into().unwrap()), 8);
    let mut leaves = [[0; 32]; 3];
    let mut seen = 0u8;
    crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload(&bytes[66..])
        .frame_fields(|name, value| {
            let index = match name {
                "position" => 0,
                "canonical" => 1,
                "original_inputs" => 2,
                unexpected => panic!("unexpected canonical frame field {unexpected}"),
            };
            assert_eq!(seen & (1 << index), 0);
            seen |= 1 << index;
            leaves[index] = Sha256::digest(value.0).into();
            Ok(())
        })
        .unwrap();
    assert_eq!(seen, 7);
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.canonical.v8\0");
    digest.update(&bytes[10..26]);
    for leaf in leaves {
        digest.update(leaf);
    }
    bytes[26..58].copy_from_slice(&digest.finalize());
    let length = (bytes.len() - 66) as u64;
    bytes[58..66].copy_from_slice(&length.to_le_bytes());
}
