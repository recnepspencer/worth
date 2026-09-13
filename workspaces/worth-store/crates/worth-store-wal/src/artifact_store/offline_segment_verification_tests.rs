use sha2::{Digest, Sha256};

use super::{
    prepare_wal_frame_append, verify_bounded_wal_segment, BoundedWalSegmentDenial,
    BoundedWalSegmentVerificationRequest,
};

const C9_WAL_V1_GOLDEN: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";

#[test]
fn owner_append_mapping_reproduces_the_frozen_wal_v1_frame() {
    let directory = tempfile::tempdir().unwrap();
    let plan = prepare_wal_frame_append(
        directory.path(),
        1,
        2,
        3,
        4,
        "c9-wal-v1-golden",
        &[0x10, 0x20, 0x30],
    )
    .unwrap();

    assert_eq!(plan.encoded_frame(), literal(C9_WAL_V1_GOLDEN));
    assert_eq!(
        plan.relative_path(),
        std::path::Path::new("wal/segment-1-generation-2.wal")
    );
    assert_eq!(plan.valid_prefix_bytes(), 0);
    assert_eq!(plan.observed_file_bytes(), 0);
}

#[test]
fn bounded_owner_decode_accepts_exact_contiguous_frame_stream() {
    let fixture = WalSegmentFixture::new();
    let observation = verify_bounded_wal_segment(&fixture.path, fixture.request(&fixture.bytes))
        .expect("canonical WAL segment");

    assert_eq!(observation.frame_count(), 2);
    assert_eq!(observation.bytes_read(), fixture.bytes.len() as u64);
    assert!(observation.peak_buffer_bytes() <= 181);
}

fn literal(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).expect("literal hex byte")
        })
        .collect()
}

#[test]
fn owner_decode_rejects_rehashed_payload_corruption() {
    let fixture = WalSegmentFixture::new();
    let mut corrupted = fixture.bytes.clone();
    corrupted[116] ^= 0x40;
    std::fs::write(&fixture.path, &corrupted).expect("mutated WAL");

    let denial = verify_bounded_wal_segment(&fixture.path, fixture.request(&corrupted))
        .expect_err("outer digest cannot replace WAL frame integrity");

    assert!(matches!(
        denial,
        BoundedWalSegmentDenial::PayloadDigestMismatch
    ));
}

struct WalSegmentFixture {
    _directory: tempfile::TempDir,
    path: std::path::PathBuf,
    bytes: Vec<u8>,
}

impl WalSegmentFixture {
    fn new() -> Self {
        let directory = tempfile::Builder::new()
            .prefix("worth-store-wal-owner-decode-")
            .tempdir()
            .expect("fixture directory");
        let first = prepare_wal_frame_append(directory.path(), 7, 3, 10, 11, "frame-a", b"first")
            .expect("first frame");
        let second = prepare_wal_frame_append(directory.path(), 7, 3, 11, 12, "frame-b", b"second")
            .expect("second frame");
        let mut bytes = first.encoded_frame().to_vec();
        bytes.extend_from_slice(second.encoded_frame());
        let path = directory.path().join("segment.wal");
        std::fs::write(&path, &bytes).expect("WAL segment");
        Self {
            _directory: directory,
            path,
            bytes,
        }
    }

    fn request(&self, bytes: &[u8]) -> BoundedWalSegmentVerificationRequest {
        BoundedWalSegmentVerificationRequest::new(
            7,
            3,
            10,
            12,
            bytes.len() as u64,
            Sha256::digest(bytes).into(),
            181,
        )
        .expect("bounded request")
    }
}
