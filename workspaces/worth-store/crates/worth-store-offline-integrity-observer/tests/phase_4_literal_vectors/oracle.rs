use worth_store_physical_format::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityFormatDeclaration,
};

#[derive(Clone, Copy)]
pub(super) enum LiteralChecksum {
    Crc32c(u32),
    Sha256([u8; 32]),
}

#[derive(Clone, Copy)]
pub(super) struct LiteralChecksumExpectation<'ranges> {
    pub(super) checksum: LiteralChecksum,
    pub(super) ranges: &'ranges [(usize, usize)],
    pub(super) field: (usize, usize),
}

pub(super) struct LiteralVector<'checksums, 'ranges> {
    pub(super) name: &'static str,
    pub(super) bytes: Vec<u8>,
    pub(super) checksums: &'checksums [LiteralChecksumExpectation<'ranges>],
}

#[derive(Clone, Copy)]
pub(super) struct DeclarationChecksumExpectation {
    pub(super) algorithm: PhysicalIntegrityAlgorithm,
    pub(super) ranges: &'static [(
        PhysicalIntegrityCoverageBoundary,
        PhysicalIntegrityCoverageBoundary,
    )],
    pub(super) field: (
        PhysicalIntegrityCoverageBoundary,
        PhysicalIntegrityCoverageBoundary,
    ),
}

pub(super) fn assert_declaration(
    declaration: PhysicalIntegrityFormatDeclaration,
    family: PhysicalIntegrityArtifactFamily,
    format_version: u16,
    envelope_schema: Option<u16>,
    expected: &[DeclarationChecksumExpectation],
) {
    assert_eq!(declaration.family(), family);
    assert_eq!(declaration.version().format_version(), format_version);
    assert_eq!(declaration.version().envelope_schema(), envelope_schema);
    assert_eq!(declaration.checksums().len(), expected.len());
    for (actual, expected) in declaration.checksums().iter().zip(expected) {
        assert_eq!(actual.algorithm(), expected.algorithm);
        assert_eq!(actual.covered_ranges().len(), expected.ranges.len());
        for (actual, expected) in actual.covered_ranges().iter().zip(expected.ranges) {
            assert_eq!((actual.start(), actual.end()), *expected);
        }
        assert_eq!(
            (actual.field().start(), actual.field().end()),
            expected.field
        );
    }
}

pub(super) fn assert_literal_vector(vector: LiteralVector<'_, '_>) {
    for expectation in vector.checksums {
        assert!(
            matches_stored_checksum(&vector.bytes, expectation, expectation.ranges),
            "{} literal checksum drift",
            vector.name
        );
        assert_covered_byte_mutation_fails(&vector, expectation);
        assert_checksum_mutation_fails(&vector, expectation);
        assert_range_mutation_fails(&vector, expectation);
    }
}

fn assert_covered_byte_mutation_fails(
    vector: &LiteralVector<'_, '_>,
    expectation: &LiteralChecksumExpectation<'_>,
) {
    for &(probe, _) in expectation.ranges {
        let mut mutated = vector.bytes.clone();
        mutated[probe] ^= 0x80;
        assert!(
            !matches_stored_checksum(&mutated, expectation, expectation.ranges),
            "{} accepted a covered-byte mutation at {probe}",
            vector.name
        );
    }
}

fn assert_checksum_mutation_fails(
    vector: &LiteralVector<'_, '_>,
    expectation: &LiteralChecksumExpectation<'_>,
) {
    let mut mutated = vector.bytes.clone();
    mutated[expectation.field.0] ^= 0x80;
    assert!(
        !matches_stored_checksum(&mutated, expectation, expectation.ranges),
        "{} accepted a checksum-field mutation",
        vector.name
    );
}

fn assert_range_mutation_fails(
    vector: &LiteralVector<'_, '_>,
    expectation: &LiteralChecksumExpectation<'_>,
) {
    for range_index in 0..expectation.ranges.len() {
        let mut wrong_ranges = expectation.ranges.to_vec();
        assert!(wrong_ranges[range_index].0 + 1 < wrong_ranges[range_index].1);
        wrong_ranges[range_index].0 += 1;
        assert!(
            !matches_stored_checksum(&vector.bytes, expectation, &wrong_ranges),
            "{} accepted shifted checksum range {range_index}",
            vector.name
        );
    }
}

fn matches_stored_checksum(
    bytes: &[u8],
    expectation: &LiteralChecksumExpectation<'_>,
    ranges: &[(usize, usize)],
) -> bool {
    let parts: Vec<&[u8]> = ranges
        .iter()
        .map(|&(start, end)| &bytes[start..end])
        .collect();
    let stored = &bytes[expectation.field.0..expectation.field.1];
    match expectation.checksum {
        LiteralChecksum::Crc32c(expected) => {
            expected.to_le_bytes() == stored && crc32c(&parts) == expected
        }
        LiteralChecksum::Sha256(expected) => expected == stored && sha256(&parts) == expected,
    }
}

pub(super) fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
                .expect("literal vector contains a hexadecimal byte")
        })
        .collect()
}

fn crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = u32::MAX;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}

pub(super) fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let bytes: Vec<u8> = parts.iter().flat_map(|part| part.iter().copied()).collect();
    sha256_bytes(&bytes)
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    const INITIAL: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let bit_length = (bytes.len() as u64).wrapping_mul(8);
    let padded_length = (bytes.len() + 9).div_ceil(64) * 64;
    let mut padded = vec![0_u8; padded_length];
    padded[..bytes.len()].copy_from_slice(bytes);
    padded[bytes.len()] = 0x80;
    padded[padded_length - 8..].copy_from_slice(&bit_length.to_be_bytes());
    let mut state = INITIAL;
    for block in padded.chunks_exact(64) {
        compress(&mut state, block);
    }
    let mut digest = [0_u8; 32];
    for (target, word) in digest.chunks_exact_mut(4).zip(state) {
        target.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn compress(state: &mut [u32; 8], block: &[u8]) {
    const ROUND: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let mut words = [0_u32; 64];
    for (index, source) in block.chunks_exact(4).enumerate() {
        words[index] = u32::from_be_bytes(source.try_into().unwrap());
    }
    for index in 16..64 {
        let first = words[index - 15].rotate_right(7)
            ^ words[index - 15].rotate_right(18)
            ^ (words[index - 15] >> 3);
        let second = words[index - 2].rotate_right(17)
            ^ words[index - 2].rotate_right(19)
            ^ (words[index - 2] >> 10);
        words[index] = words[index - 16]
            .wrapping_add(first)
            .wrapping_add(words[index - 7])
            .wrapping_add(second);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for (word, round) in words.into_iter().zip(ROUND) {
        let choose = (e & f) ^ ((!e) & g);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let upper_a = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let upper_e = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let first = h
            .wrapping_add(upper_e)
            .wrapping_add(choose)
            .wrapping_add(round)
            .wrapping_add(word);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(first);
        d = c;
        c = b;
        b = a;
        a = first.wrapping_add(upper_a.wrapping_add(majority));
    }
    for (target, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *target = target.wrapping_add(value);
    }
}
