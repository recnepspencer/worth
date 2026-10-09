use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use worth_store_physical_format::BTreeNodeV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutIndexBackupFormat {
    BTreeNodeV1,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundedLayoutIndexVerificationRequest<'a> {
    format: LayoutIndexBackupFormat,
    expected_identity: &'a str,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
}

impl<'a> BoundedLayoutIndexVerificationRequest<'a> {
    pub const fn new(
        format: LayoutIndexBackupFormat,
        expected_identity: &'a str,
        expected_bytes: u64,
        expected_digest: [u8; 32],
        max_buffer_bytes: usize,
    ) -> Self {
        Self {
            format,
            expected_identity,
            expected_bytes,
            expected_digest,
            max_buffer_bytes,
        }
    }
}

#[derive(Debug)]
pub enum BoundedLayoutIndexDenial {
    Io(std::io::Error),
    BufferTooSmall { required: usize, actual: usize },
    LengthMismatch { expected: u64, actual: u64 },
    DigestMismatch,
    IdentityMismatch,
    MalformedIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedLayoutIndexObservation {
    bytes_read: u64,
    peak_buffer_bytes: u64,
}

impl BoundedLayoutIndexObservation {
    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }

    pub const fn peak_buffer_bytes(self) -> u64 {
        self.peak_buffer_bytes
    }
}

pub fn verify_bounded_layout_index_artifact(
    path: &Path,
    request: BoundedLayoutIndexVerificationRequest<'_>,
) -> Result<BoundedLayoutIndexObservation, BoundedLayoutIndexDenial> {
    let mut file = std::fs::File::open(path).map_err(BoundedLayoutIndexDenial::Io)?;
    let actual = file.metadata().map_err(BoundedLayoutIndexDenial::Io)?.len();
    verify_bounded_layout_index_artifact_from_reader(&mut file, actual, request)
}

pub fn verify_bounded_layout_index_artifact_from_reader(
    reader: &mut impl Read,
    actual: u64,
    request: BoundedLayoutIndexVerificationRequest<'_>,
) -> Result<BoundedLayoutIndexObservation, BoundedLayoutIndexDenial> {
    const MAXIMUM_BTREE_NODE_BYTES: usize = 65_424;
    let required = usize::try_from(request.expected_bytes).map_err(|_| {
        BoundedLayoutIndexDenial::LengthMismatch {
            expected: MAXIMUM_BTREE_NODE_BYTES as u64,
            actual: request.expected_bytes,
        }
    })?;
    if !(104..=MAXIMUM_BTREE_NODE_BYTES).contains(&required) {
        return Err(BoundedLayoutIndexDenial::LengthMismatch {
            expected: MAXIMUM_BTREE_NODE_BYTES as u64,
            actual: request.expected_bytes,
        });
    }
    if request.max_buffer_bytes < required {
        return Err(BoundedLayoutIndexDenial::BufferTooSmall {
            required,
            actual: request.max_buffer_bytes,
        });
    }
    if actual != request.expected_bytes {
        return Err(BoundedLayoutIndexDenial::LengthMismatch {
            expected: request.expected_bytes,
            actual,
        });
    }
    let mut bytes = vec![0_u8; required];
    reader
        .read_exact(&mut bytes)
        .map_err(BoundedLayoutIndexDenial::Io)?;
    let observed_digest: [u8; 32] = Sha256::digest(&bytes).into();
    if observed_digest != request.expected_digest {
        return Err(BoundedLayoutIndexDenial::DigestMismatch);
    }
    if !identity_names_digest(request.expected_identity, observed_digest) {
        return Err(BoundedLayoutIndexDenial::IdentityMismatch);
    }
    let valid = match request.format {
        LayoutIndexBackupFormat::BTreeNodeV1 => BTreeNodeV1::decode(&bytes).is_ok(),
    };
    if !valid {
        return Err(BoundedLayoutIndexDenial::MalformedIndex);
    }
    Ok(BoundedLayoutIndexObservation {
        bytes_read: required as u64,
        peak_buffer_bytes: required as u64,
    })
}

fn identity_names_digest(identity: &str, digest: [u8; 32]) -> bool {
    const PREFIX: &[u8] = b"index:sha256:";
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = identity.as_bytes();
    if bytes.len() != PREFIX.len() + 64 || !bytes.starts_with(PREFIX) {
        return false;
    }
    digest.iter().enumerate().all(|(index, byte)| {
        bytes[PREFIX.len() + index * 2] == HEX[(byte >> 4) as usize]
            && bytes[PREFIX.len() + index * 2 + 1] == HEX[(byte & 0x0f) as usize]
    })
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use worth_store_physical_format::{BTreeNodeCellV1, BTreeNodeV1};

    use super::*;

    #[test]
    fn bounded_layout_owner_decode_rejects_wrong_node_version() {
        let mut bytes = node_bytes(1);
        bytes[8] = 2;
        let identity = format!("index:sha256:{}", hex(&Sha256::digest(&bytes)));
        let file = temporary_index_file();
        let path = file.path();
        std::fs::write(path, &bytes).expect("index bytes");
        let denial = verify_bounded_layout_index_artifact(
            path,
            BoundedLayoutIndexVerificationRequest::new(
                LayoutIndexBackupFormat::BTreeNodeV1,
                &identity,
                bytes.len() as u64,
                Sha256::digest(&bytes).into(),
                4096,
            ),
        )
        .expect_err("outer digest cannot legalize an unsupported node version");

        assert!(matches!(denial, BoundedLayoutIndexDenial::MalformedIndex));
    }

    #[test]
    fn transport_rehash_cannot_rename_a_substituted_index() {
        let original = node_bytes(1);
        let original_identity = format!("index:sha256:{}", hex(&Sha256::digest(&original)));
        let substituted = node_bytes(2);
        let substituted_digest: [u8; 32] = Sha256::digest(&substituted).into();
        let file = temporary_index_file();
        let path = file.path();
        std::fs::write(path, &substituted).expect("substituted index bytes");

        let denial = verify_bounded_layout_index_artifact(
            path,
            BoundedLayoutIndexVerificationRequest::new(
                LayoutIndexBackupFormat::BTreeNodeV1,
                &original_identity,
                substituted.len() as u64,
                substituted_digest,
                4096,
            ),
        )
        .expect_err("a valid substitute must not inherit another index identity");

        assert!(matches!(denial, BoundedLayoutIndexDenial::IdentityMismatch));
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn node_bytes(value: u8) -> Vec<u8> {
        BTreeNodeV1::leaf(
            1,
            vec![BTreeNodeCellV1::leaf(b"catalog-key".to_vec(), vec![value])],
            None,
            None,
        )
        .expect("node")
        .encode(4096)
        .expect("encoded node")
    }

    fn temporary_index_file() -> tempfile::NamedTempFile {
        tempfile::Builder::new()
            .prefix("worth-store-layout-index-")
            .suffix(".bin")
            .tempfile()
            .unwrap()
    }
}
