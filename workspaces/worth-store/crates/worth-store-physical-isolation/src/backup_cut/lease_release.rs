use super::BackupReachabilityLease;
use worth_foundational::CanonicalDigestId;

const MAGIC: &[u8; 4] = b"WBR1";
const RECORD_BYTES: usize = 36;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReachabilityLeaseReleaseRecord {
    cut_identity: CanonicalDigestId,
    encoded: [u8; RECORD_BYTES],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBackupReachabilityLeaseReleaseRecord;

impl BackupReachabilityLeaseReleaseRecord {
    pub(crate) fn from_lease(lease: &BackupReachabilityLease) -> Self {
        let cut_identity = lease.cut_identity();
        let mut encoded = [0; RECORD_BYTES];
        encoded[..MAGIC.len()].copy_from_slice(MAGIC);
        encoded[MAGIC.len()..].copy_from_slice(&cut_identity);
        Self {
            cut_identity: CanonicalDigestId::new(cut_identity),
            encoded,
        }
    }

    pub fn recover(encoded: &[u8]) -> Result<Self, InvalidBackupReachabilityLeaseReleaseRecord> {
        if encoded.len() != RECORD_BYTES || &encoded[..MAGIC.len()] != MAGIC {
            return Err(InvalidBackupReachabilityLeaseReleaseRecord);
        }
        let cut_identity = encoded[MAGIC.len()..]
            .try_into()
            .map_err(|_| InvalidBackupReachabilityLeaseReleaseRecord)?;
        let encoded = encoded
            .try_into()
            .map_err(|_| InvalidBackupReachabilityLeaseReleaseRecord)?;
        Ok(Self {
            cut_identity: CanonicalDigestId::new(cut_identity),
            encoded,
        })
    }

    pub const fn cut_identity(&self) -> [u8; 32] {
        *self.cut_identity.bytes()
    }

    pub fn recovery_bytes(&self) -> &[u8] {
        &self.encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_release_record_is_not_owner_authority() {
        assert_eq!(
            BackupReachabilityLeaseReleaseRecord::recover(&[0; RECORD_BYTES]),
            Err(InvalidBackupReachabilityLeaseReleaseRecord)
        );
    }
}
