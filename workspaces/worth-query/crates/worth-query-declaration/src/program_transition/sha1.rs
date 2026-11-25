//! Minimal zero-dependency FIPS 180-4 SHA-1 and RFC 4122 §4.3 UUIDv5
//! derivation for deterministic package-default build identities.

/// Compute the 20-byte FIPS 180-4 SHA-1 digest of `input`.
pub(super) fn sha1_digest(input: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xefcd_ab89,
        0x98ba_dcfe,
        0x1032_5476,
        0xc3d2_e1f0,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(((input.len() + 9 + 63) / 64) * 64);
    padded.extend_from_slice(input);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for block in padded.chunks_exact(64) {
        compress_block(&mut h, block);
    }

    let mut out = [0u8; 20];
    for (index, word) in h.into_iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn compress_block(h: &mut [u32; 5], block: &[u8]) {
    let mut w = [0u32; 80];
    for (index, chunk) in block.chunks_exact(4).enumerate() {
        w[index] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    for index in 16..80 {
        w[index] = (w[index - 3] ^ w[index - 8] ^ w[index - 14] ^ w[index - 16]).rotate_left(1);
    }

    let [mut a, mut b, mut c, mut d, mut e] = *h;
    for (index, &word) in w.iter().enumerate() {
        let (f, k) = match index {
            0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999),
            20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
            _ => (b ^ c ^ d, 0xca62_c1d6),
        };
        let temp = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
}

/// Derive an RFC 4122 §4.3 UUIDv5 from a 16-byte namespace UUID and `name`.
pub(super) fn uuid_v5(namespace: [u8; 16], name: &[u8]) -> [u8; 16] {
    let mut message = Vec::with_capacity(16 + name.len());
    message.extend_from_slice(&namespace);
    message.extend_from_slice(name);
    let digest = sha1_digest(&message);

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    bytes
}
