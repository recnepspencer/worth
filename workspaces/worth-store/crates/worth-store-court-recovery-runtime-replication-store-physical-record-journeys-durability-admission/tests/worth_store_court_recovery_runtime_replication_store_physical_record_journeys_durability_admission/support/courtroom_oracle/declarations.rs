pub(crate) use {std::io::Write};
pub(crate) use {std::path::Path};

pub(crate) use {sha2::Digest, sha2::Sha256};

#[derive(Clone, Copy)]
pub(crate) struct OracleRecord {
    pub(crate) payload_bytes: usize,
    pub(crate) byte: u8,
}

pub(crate) fn seal(path: &Path) -> Vec<OracleRecord> {
    let records = records();
    let mut output = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    for record in &records {
        writeln!(output, "{},{}", record.payload_bytes, record.byte).unwrap();
    }
    output.flush().unwrap();
    records
}

pub(crate) fn read(path: &Path) -> Vec<OracleRecord> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| {
            let (payload_bytes, byte) = line.split_once(',').unwrap();
            OracleRecord {
                payload_bytes: payload_bytes.parse().unwrap(),
                byte: byte.parse().unwrap(),
            }
        })
        .collect()
}

pub(crate) fn point_digest(locator_path: &Path, records: &[OracleRecord]) -> String {
    let locators = locator_bytes(locator_path);
    assert_eq!(locators.len(), records.len());
    let mut digest = sha2::Sha256::new();
    for (locator, record) in locators.iter().zip(records) {
        digest.update(locator);
        digest.update((record.payload_bytes as u64).to_le_bytes());
        let mut payload_digest = sha2::Sha256::new();
        update_repeated(&mut payload_digest, record.byte, record.payload_bytes);
        digest.update(payload_digest.finalize());
    }
    hex(&digest.finalize())
}

pub(crate) fn scan_digest(locator_path: &Path, records: &[OracleRecord]) -> String {
    let mut rows = locator_bytes(locator_path)
        .into_iter()
        .zip(records.iter().copied())
        .collect::<Vec<_>>();
    rows.sort_by(compare_record_identity);
    let mut digest = sha2::Sha256::new();
    for (locator, record) in rows {
        digest.update(&locator[16..]);
        digest.update((record.payload_bytes as u64).to_le_bytes());
    }
    hex(&digest.finalize())
}

pub(crate) fn payload_digest(locator_path: &Path, records: &[OracleRecord]) -> String {
    let mut rows = locator_bytes(locator_path)
        .into_iter()
        .zip(records.iter().copied())
        .collect::<Vec<_>>();
    rows.sort_by(compare_record_identity);
    let mut digest = sha2::Sha256::new();
    for (_, record) in rows {
        digest.update((record.payload_bytes as u64).to_le_bytes());
        update_repeated(&mut digest, record.byte, record.payload_bytes);
    }
    hex(&digest.finalize())
}

pub(crate) fn update_repeated(digest: &mut Sha256, byte: u8, mut remaining: usize) {
    let buffer = [byte; 8_192];
    while remaining != 0 {
        let width = remaining.min(buffer.len());
        digest.update(&buffer[..width]);
        remaining -= width;
    }
}

pub(crate) fn locator_count(path: &Path) -> usize {
    std::fs::read_to_string(path).unwrap().lines().count()
}

pub(crate) fn records() -> Vec<OracleRecord> {
    let mut records = (0..1_400_u64)
        .map(|ordinal| OracleRecord {
            payload_bytes: 3_000,
            byte: (ordinal % 251) as u8,
        })
        .collect::<Vec<_>>();
    records.push(OracleRecord {
        payload_bytes: 16_384,
        byte: 0xea,
    });
    records.push(OracleRecord {
        payload_bytes: 17 * 65_536 + 7,
        byte: 0xfb,
    });
    records
}

pub(crate) fn locator_bytes(path: &Path) -> Vec<[u8; 40]> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| {
            let mut bytes = [0_u8; 40];
            for (index, pair) in line.as_bytes().chunks_exact(2).enumerate() {
                bytes[index] = u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap();
            }
            bytes
        })
        .collect()
}

pub(crate) fn compare_record_identity(
    left: &([u8; 40], OracleRecord),
    right: &([u8; 40], OracleRecord),
) -> std::cmp::Ordering {
    let left_ordinal = u64::from_le_bytes(left.0[32..40].try_into().unwrap());
    let right_ordinal = u64::from_le_bytes(right.0[32..40].try_into().unwrap());
    left.0[16..32]
        .cmp(&right.0[16..32])
        .then_with(|| left_ordinal.cmp(&right_ordinal))
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}