use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) const IDENTITY: &str = "namespace/identity";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) enum Operator {
    Clean,
    CoveredByte,
    ChecksumByte,
    Truncate,
    Remove,
    Encoding,
    Schema,
}

impl Operator {
    pub(super) const ALL: [Self; 7] = [
        Self::Clean,
        Self::CoveredByte,
        Self::ChecksumByte,
        Self::Truncate,
        Self::Remove,
        Self::Encoding,
        Self::Schema,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::CoveredByte => "B-covered-byte",
            Self::ChecksumByte => "K-checksum-byte",
            Self::Truncate => "T-truncated-tail",
            Self::Remove => "R-removed-identity",
            Self::Encoding => "U-encoding",
            Self::Schema => "U-namespace-schema",
        }
    }
}

pub(super) fn edit(root: &Path, operator: Operator) {
    let path = root.join(IDENTITY);
    let mut bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.len(), 72);
    match operator {
        Operator::Clean => return,
        Operator::CoveredByte => bytes[24] ^= 1,
        Operator::ChecksumByte => bytes[40] ^= 1,
        Operator::Truncate => {
            bytes.pop();
        }
        Operator::Remove => {
            std::fs::remove_file(path).unwrap();
            return;
        }
        Operator::Encoding | Operator::Schema => {
            let offset = if matches!(operator, Operator::Encoding) {
                8
            } else {
                10
            };
            bytes[offset..offset + 2].copy_from_slice(&2_u16.to_le_bytes());
            let digest = Sha256::digest(&bytes[..40]);
            bytes[40..72].copy_from_slice(&digest);
        }
    }
    write(&path, &bytes);
}

pub(super) fn write(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

pub(super) fn audit(root: &Path, pristine: &[u8], operator: Operator) {
    if matches!(operator, Operator::Remove) {
        assert_eq!(
            std::fs::symlink_metadata(root.join(IDENTITY))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::NotFound
        );
        return;
    }
    let actual = std::fs::read(root.join(IDENTITY)).unwrap();
    match operator {
        Operator::Clean => assert_eq!(actual, pristine),
        Operator::CoveredByte | Operator::ChecksumByte => {
            let changed = if matches!(operator, Operator::CoveredByte) {
                24
            } else {
                40
            };
            assert_eq!(actual.len(), pristine.len());
            for index in 0..72 {
                assert_eq!(actual[index] ^ pristine[index], u8::from(index == changed));
            }
        }
        Operator::Truncate => assert_eq!(actual, pristine[..71]),
        Operator::Encoding | Operator::Schema => {
            let offset = if matches!(operator, Operator::Encoding) {
                8
            } else {
                10
            };
            assert_eq!(actual.len(), 72);
            assert_eq!(&actual[..offset], &pristine[..offset]);
            assert_eq!(&actual[offset..offset + 2], &[2, 0]);
            assert_eq!(&actual[offset + 2..40], &pristine[offset + 2..40]);
            assert_eq!(&actual[40..72], &Sha256::digest(&actual[..40])[..]);
        }
        Operator::Remove => unreachable!(),
    }
}
