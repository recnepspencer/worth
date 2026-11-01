//! Pure framing meaning only: no execution request, runtime or recovery authority.
use super::*;

fn literal_empty_frame() -> [u8; 52] {
    [
        87, 87, 82, 76, 68, 67, 80, 49, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xe3, 0xb0, 0xc4, 0x42,
        0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24, 0x27, 0xae, 0x41,
        0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
    ]
}
#[test]
fn literal_frame_checks_magic_version_length_and_payload_digest() {
    let bytes = literal_empty_frame();
    assert!(inspect(&bytes, &mut |_| Ok(())).unwrap().is_empty());
    let mut damaged = bytes;
    damaged[0] ^= 1;
    assert!(matches!(
        inspect(&damaged, &mut |_| Ok(())),
        Err(Denial::Frame(Frame::Malformed))
    ));
    let mut newer = bytes;
    newer[8] = 2;
    assert!(matches!(
        inspect(&newer, &mut |_| Ok(())),
        Err(Denial::Frame(Frame::UnsupportedVersion(_)))
    ));
    let mut length = bytes;
    length[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        inspect(&length, &mut |_| Ok(())),
        Err(Denial::Frame(Frame::Malformed))
    ));
    let mut digest = bytes;
    digest[20] ^= 1;
    assert!(matches!(
        inspect(&digest, &mut |_| Ok(())),
        Err(Denial::Frame(Frame::DigestMismatch))
    ));
}
#[test]
fn checksum_visits_bounded_chunks_and_returns_callback_refusal() {
    let payload = vec![7; (64 << 10) + 1];
    let mut bytes = literal_empty_frame().to_vec();
    bytes[12..20].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes[20..52].copy_from_slice(&Sha256::digest(&payload));
    bytes.extend_from_slice(&payload);
    let mut chunks = Vec::new();
    assert_eq!(
        inspect(&bytes, &mut |size| {
            chunks.push(size);
            Ok(())
        })
        .unwrap(),
        payload
    );
    assert_eq!(chunks, [64 << 10, 1]);
    let mut calls = 0;
    let refused = inspect(&bytes, &mut |_| {
        calls += 1;
        if calls == 2 {
            Err(Denial::PublicationBusy)
        } else {
            Ok(())
        }
    });
    assert!(matches!(refused, Err(Denial::PublicationBusy)));
    assert_eq!(calls, 2);
}
