const HEADER_BYTES: usize = 32;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ActivationField {
    TenantScope,
    KeyScope,
    CanonicalKey,
    SelectedVersion,
    SelectedValueSequence,
    SelectedPublicationSequence,
    SelectedTombstoneSequence,
    OutputSequence,
    Segment,
    Generation,
    LsnStart,
    LsnEnd,
    FrameDigest,
    ExpectedBytes,
    OutputPath,
    OutputOffset,
    OutputBytes,
    CheckpointEpoch,
    CoverageStart,
    CoverageEnd,
    ManifestDigest,
}

pub(crate) const ALL_ACTIVATION_FIELDS: [ActivationField; 21] = [
    ActivationField::TenantScope,
    ActivationField::KeyScope,
    ActivationField::CanonicalKey,
    ActivationField::SelectedVersion,
    ActivationField::SelectedValueSequence,
    ActivationField::SelectedPublicationSequence,
    ActivationField::SelectedTombstoneSequence,
    ActivationField::OutputSequence,
    ActivationField::Segment,
    ActivationField::Generation,
    ActivationField::LsnStart,
    ActivationField::LsnEnd,
    ActivationField::FrameDigest,
    ActivationField::ExpectedBytes,
    ActivationField::OutputPath,
    ActivationField::OutputOffset,
    ActivationField::OutputBytes,
    ActivationField::CheckpointEpoch,
    ActivationField::CoverageStart,
    ActivationField::CoverageEnd,
    ActivationField::ManifestDigest,
];

pub(crate) fn mutate_activation_field(bytes: &[u8], field: ActivationField) -> Vec<u8> {
    let mut mutated = bytes.to_vec();
    let mut cursor = Cursor::new(&mutated[HEADER_BYTES..]);
    let tenant = cursor.u8();
    let key_scope = cursor.u8();
    let canonical_key = cursor.blob();
    let selected_version = cursor.u64();
    let selected_value_sequence = cursor.u64();
    cursor.u8();
    let selected_publication_sequence = cursor.u64();
    cursor.u8();
    let selected_tombstone_sequence = cursor.u64();
    cursor.u8();
    let selected_base = cursor.u8();
    assert_eq!(mutated[HEADER_BYTES + selected_base.start], 0);
    let output_sequence = cursor.u64();
    cursor.u8();
    let segment = cursor.u64();
    let generation = cursor.u64();
    let lsn_start = cursor.u64();
    let lsn_end = cursor.u64();
    let frame_digest = cursor.blob();
    let expected_bytes = cursor.u64();
    let output_path = cursor.blob();
    let output_offset = cursor.u64();
    let output_bytes = cursor.u64();
    let checkpoint_epoch = cursor.u64();
    let coverage_start = cursor.u64();
    let coverage_end = cursor.u64();
    let manifest_digest = cursor.blob();
    assert_eq!(cursor.position(), mutated.len() - HEADER_BYTES);

    let payload_range = match field {
        ActivationField::TenantScope => tenant,
        ActivationField::KeyScope => key_scope,
        ActivationField::CanonicalKey => canonical_key,
        ActivationField::SelectedVersion => selected_version,
        ActivationField::SelectedValueSequence => selected_value_sequence,
        ActivationField::SelectedPublicationSequence => selected_publication_sequence,
        ActivationField::SelectedTombstoneSequence => selected_tombstone_sequence,
        ActivationField::OutputSequence => output_sequence,
        ActivationField::Segment => segment,
        ActivationField::Generation => generation,
        ActivationField::LsnStart => lsn_start,
        ActivationField::LsnEnd => lsn_end,
        ActivationField::FrameDigest => frame_digest,
        ActivationField::ExpectedBytes => expected_bytes,
        ActivationField::OutputPath => output_path,
        ActivationField::OutputOffset => output_offset,
        ActivationField::OutputBytes => output_bytes,
        ActivationField::CheckpointEpoch => checkpoint_epoch,
        ActivationField::CoverageStart => coverage_start,
        ActivationField::CoverageEnd => coverage_end,
        ActivationField::ManifestDigest => manifest_digest,
    };
    let range = (HEADER_BYTES + payload_range.start)..(HEADER_BYTES + payload_range.end);
    mutate_value(&mut mutated[range], field);
    rewrite_payload_checksum(&mut mutated);
    mutated
}

pub(crate) fn corrupt_persisted_byte(path: &std::path::Path, offset: u64) {
    use std::io::{Read, Seek, SeekFrom, Write};
    let mut artifact = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    artifact.seek(SeekFrom::Start(offset)).unwrap();
    let mut byte = [0];
    artifact.read_exact(&mut byte).unwrap();
    byte[0] ^= 0x01;
    artifact.seek(SeekFrom::Start(offset)).unwrap();
    artifact.write_all(&byte).unwrap();
    artifact.sync_all().unwrap();
}

fn mutate_value(value: &mut [u8], field: ActivationField) {
    match field {
        ActivationField::TenantScope => value[0] = (value[0] + 1) % 7,
        ActivationField::KeyScope => value[0] = (value[0] + 1) % 9,
        ActivationField::CanonicalKey
        | ActivationField::FrameDigest
        | ActivationField::OutputPath
        | ActivationField::ManifestDigest => value[0] ^= 1,
        _ => {
            let current = u64::from_le_bytes(value.try_into().expect("u64 activation field"));
            value.copy_from_slice(&current.wrapping_add(1).to_le_bytes());
        }
    }
}

fn rewrite_payload_checksum(bytes: &mut [u8]) {
    let version = &bytes[8..10];
    let payload_len = &bytes[12..16];
    let mut protected = Vec::with_capacity(6 + bytes.len() - HEADER_BYTES);
    protected.extend_from_slice(version);
    protected.extend_from_slice(payload_len);
    protected.extend_from_slice(&bytes[HEADER_BYTES..]);
    bytes[24..32].copy_from_slice(&checksum(&protected).to_le_bytes());
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

struct Cursor<'a> {
    payload: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(payload: &'a [u8]) -> Self {
        Self { payload, offset: 0 }
    }

    const fn position(&self) -> usize {
        self.offset
    }

    fn u8(&mut self) -> std::ops::Range<usize> {
        self.take(1)
    }

    fn u64(&mut self) -> std::ops::Range<usize> {
        self.take(8)
    }

    fn blob(&mut self) -> std::ops::Range<usize> {
        let length_offset = self.offset;
        let len = u32::from_le_bytes(
            self.payload[length_offset..length_offset + 4]
                .try_into()
                .expect("blob length"),
        ) as usize;
        self.offset += 4;
        self.take(len)
    }

    fn take(&mut self, len: usize) -> std::ops::Range<usize> {
        let start = self.offset;
        self.offset += len;
        start..self.offset
    }
}
