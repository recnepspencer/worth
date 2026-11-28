//! The current World checkpoint uses the canonical empty frame.
use super::*;
use worth_execution::inspect_framed_envelope;

fn inspect(bytes: &[u8]) -> Result<&[u8], Denial> {
    inspect_framed_envelope(SPEC, bytes, |_| Ok::<_, FramedEnvelopeError>(()))
        .map(|(_, payload)| payload)
        .map_err(frame_denial)
}

fn literal_empty_frame() -> [u8; 52] {
    [
        87, 87, 82, 76, 68, 67, 80, 49, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xe3, 0xb0, 0xc4, 0x42,
        0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24, 0x27, 0xae, 0x41,
        0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
    ]
}
#[test]
fn literal_frame_checks_magic_version_length_and_payload_digest() {
    let bytes = literal_empty_frame();
    assert!(inspect(&bytes).unwrap().is_empty());
    let mut damaged = bytes;
    damaged[0] ^= 1;
    assert!(matches!(
        inspect(&damaged),
        Err(Denial::Frame(Frame::Malformed))
    ));
    let mut newer = bytes;
    newer[8] = 4;
    assert!(matches!(
        inspect(&newer),
        Err(Denial::Frame(Frame::UnsupportedVersion(_)))
    ));
    for version in [1u32, 2] {
        let mut older = bytes;
        older[8..12].copy_from_slice(&version.to_le_bytes());
        assert!(matches!(
            inspect(&older),
            Err(Denial::Frame(Frame::UnsupportedVersion(_)))
        ));
    }
    let mut length = bytes;
    length[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        inspect(&length),
        Err(Denial::Frame(Frame::Malformed))
    ));
    let mut digest = bytes;
    digest[20] ^= 1;
    assert!(matches!(
        inspect(&digest),
        Err(Denial::Frame(Frame::DigestMismatch))
    ));
}
