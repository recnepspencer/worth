//! Runtime-dispatched production SHA-256 equals a forced-software oracle.
use sha2::{Digest, Sha256};

#[test]
fn sha256_backend_preserves_bytes_across_padding_chunking_and_corpus() {
    // A different package version keeps force-soft from unifying onto the
    // production dependency. This oracle cannot disable production hardware.
    use sha2_software::Digest as SoftwareDigest;
    let mut state = 0x5eed_95_u64;
    for length in [
        0, 1, 7, 31, 55, 56, 63, 64, 65, 111, 112, 127, 128, 129, 511, 512, 4096, 16383,
    ] {
        let bytes = (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect::<Vec<_>>();
        let expected: [u8; 32] = sha2_software::Sha256::digest(&bytes).into();
        assert_eq!(<[u8; 32]>::from(Sha256::digest(&bytes)), expected);
        for width in [1, 13, 64, 67, 1024] {
            let mut production = Sha256::new();
            for chunk in bytes.chunks(width) {
                production.update(chunk);
            }
            assert_eq!(
                <[u8; 32]>::from(production.finalize()),
                expected,
                "length={length}, chunk width={width}"
            );
        }
    }
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(b"abc")),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad
        ]
    );
}
