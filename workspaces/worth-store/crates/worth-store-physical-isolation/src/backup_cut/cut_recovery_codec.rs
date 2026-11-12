use sha2::{Digest, Sha256};
use worth_foundational::CanonicalDigestId;

use super::cut_recovery::BackupCutRecoverySource;
use super::cut_recovery_path::{
    decode_recovery_path, encode_recovery_path, RecoveryPathCodecDenial,
};

const MAGIC: &[u8; 4] = b"WBC1";
const CHECKSUM_BYTES: usize = 32;
pub(super) const MAX_BACKUP_CUT_RECOVERY_BYTES: usize = 64 * 1024 * 1024;
const MAX_RECOVERY_SOURCES: usize = 262_144;

pub(super) struct DecodedBackupCutRecovery {
    pub(super) authority_fingerprint: CanonicalDigestId,
    pub(super) security_progression_fingerprint: u64,
    pub(super) format_profile: String,
    pub(super) backend_profile: String,
    pub(super) manifest_bytes: Vec<u8>,
    pub(super) sources: Vec<BackupCutRecoverySource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BackupCutRecoveryCodecDenial {
    InvalidEncoding,
    UnsupportedPathPlatform,
    AllocationFailed,
    SizeLimitExceeded,
}

pub(super) fn encode_backup_cut_recovery(
    authority_fingerprint: [u8; 32],
    security_progression_fingerprint: u64,
    format_profile: &str,
    backend_profile: &str,
    manifest_bytes: &[u8],
    sources: &[BackupCutRecoverySource],
) -> Result<Vec<u8>, BackupCutRecoveryCodecDenial> {
    if sources.len() > MAX_RECOVERY_SOURCES {
        return Err(BackupCutRecoveryCodecDenial::SizeLimitExceeded);
    }
    let mut encoded = Vec::new();
    push_raw(&mut encoded, MAGIC)?;
    push_raw(&mut encoded, &authority_fingerprint)?;
    push_raw(
        &mut encoded,
        &security_progression_fingerprint.to_le_bytes(),
    )?;
    push_bytes(&mut encoded, format_profile.as_bytes())?;
    push_bytes(&mut encoded, backend_profile.as_bytes())?;
    push_bytes(&mut encoded, manifest_bytes)?;
    push_u32(&mut encoded, sources.len())?;
    for source in sources {
        push_bytes(&mut encoded, source.output_name.as_bytes())?;
        push_raw(&mut encoded, source.physical_identity.bytes())?;
        let (platform, path) = encode_recovery_path(&source.path).map_err(map_path_denial)?;
        push_raw(&mut encoded, &[platform])?;
        push_bytes(&mut encoded, &path)?;
    }
    let checksum = Sha256::digest(&encoded);
    push_raw(&mut encoded, &checksum)?;
    Ok(encoded)
}

pub(super) fn decode_backup_cut_recovery(
    encoded: &[u8],
) -> Result<DecodedBackupCutRecovery, BackupCutRecoveryCodecDenial> {
    if encoded.len() < MAGIC.len() + 32 + 8 + CHECKSUM_BYTES {
        return Err(BackupCutRecoveryCodecDenial::InvalidEncoding);
    }
    if encoded.len() > MAX_BACKUP_CUT_RECOVERY_BYTES {
        return Err(BackupCutRecoveryCodecDenial::SizeLimitExceeded);
    }
    let (payload, checksum) = encoded.split_at(encoded.len() - CHECKSUM_BYTES);
    if Sha256::digest(payload).as_slice() != checksum || &payload[..MAGIC.len()] != MAGIC {
        return Err(BackupCutRecoveryCodecDenial::InvalidEncoding);
    }
    let mut cursor = Cursor::new(&payload[MAGIC.len()..]);
    let authority_fingerprint = cursor.take_array()?;
    let security_progression_fingerprint = cursor.take_u64()?;
    let format_profile = cursor.take_string()?;
    let backend_profile = cursor.take_string()?;
    let manifest_bytes = copy_bytes(cursor.take_bytes()?)?;
    let source_count = cursor.take_u32()? as usize;
    if source_count > MAX_RECOVERY_SOURCES {
        return Err(BackupCutRecoveryCodecDenial::SizeLimitExceeded);
    }
    let mut sources = Vec::new();
    sources
        .try_reserve_exact(source_count)
        .map_err(|_| BackupCutRecoveryCodecDenial::AllocationFailed)?;
    for _ in 0..source_count {
        let output_name = cursor.take_string()?;
        let physical_identity = cursor.take_array()?;
        let platform = cursor.take_u8()?;
        let path =
            decode_recovery_path(platform, cursor.take_bytes()?).map_err(
                |denial| match denial {
                    RecoveryPathCodecDenial::InvalidEncoding => {
                        BackupCutRecoveryCodecDenial::InvalidEncoding
                    }
                    RecoveryPathCodecDenial::UnsupportedPlatform => {
                        BackupCutRecoveryCodecDenial::UnsupportedPathPlatform
                    }
                    RecoveryPathCodecDenial::AllocationFailed => {
                        BackupCutRecoveryCodecDenial::AllocationFailed
                    }
                },
            )?;
        if output_name.is_empty() || path.as_os_str().is_empty() {
            return Err(BackupCutRecoveryCodecDenial::InvalidEncoding);
        }
        sources.push(BackupCutRecoverySource {
            output_name,
            path,
            physical_identity: CanonicalDigestId::new(physical_identity),
        });
    }
    if !cursor.is_empty() {
        return Err(BackupCutRecoveryCodecDenial::InvalidEncoding);
    }
    Ok(DecodedBackupCutRecovery {
        authority_fingerprint: CanonicalDigestId::new(authority_fingerprint),
        security_progression_fingerprint,
        format_profile,
        backend_profile,
        manifest_bytes,
        sources,
    })
}

fn push_bytes(encoded: &mut Vec<u8>, bytes: &[u8]) -> Result<(), BackupCutRecoveryCodecDenial> {
    push_u32(encoded, bytes.len())?;
    push_raw(encoded, bytes)
}

fn push_u32(encoded: &mut Vec<u8>, value: usize) -> Result<(), BackupCutRecoveryCodecDenial> {
    let value = u32::try_from(value).map_err(|_| BackupCutRecoveryCodecDenial::InvalidEncoding)?;
    push_raw(encoded, &value.to_le_bytes())
}

fn push_raw(encoded: &mut Vec<u8>, bytes: &[u8]) -> Result<(), BackupCutRecoveryCodecDenial> {
    let required = encoded
        .len()
        .checked_add(bytes.len())
        .ok_or(BackupCutRecoveryCodecDenial::SizeLimitExceeded)?;
    if required > MAX_BACKUP_CUT_RECOVERY_BYTES {
        return Err(BackupCutRecoveryCodecDenial::SizeLimitExceeded);
    }
    encoded
        .try_reserve_exact(bytes.len())
        .map_err(|_| BackupCutRecoveryCodecDenial::AllocationFailed)?;
    encoded.extend_from_slice(bytes);
    Ok(())
}

fn copy_bytes(bytes: &[u8]) -> Result<Vec<u8>, BackupCutRecoveryCodecDenial> {
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(bytes.len())
        .map_err(|_| BackupCutRecoveryCodecDenial::AllocationFailed)?;
    copied.extend_from_slice(bytes);
    Ok(copied)
}

const fn map_path_denial(denial: RecoveryPathCodecDenial) -> BackupCutRecoveryCodecDenial {
    match denial {
        RecoveryPathCodecDenial::InvalidEncoding => BackupCutRecoveryCodecDenial::InvalidEncoding,
        RecoveryPathCodecDenial::UnsupportedPlatform => {
            BackupCutRecoveryCodecDenial::UnsupportedPathPlatform
        }
        RecoveryPathCodecDenial::AllocationFailed => BackupCutRecoveryCodecDenial::AllocationFailed,
    }
}

struct Cursor<'a> {
    remaining: &'a [u8],
}

impl<'a> Cursor<'a> {
    const fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], BackupCutRecoveryCodecDenial> {
        if count > self.remaining.len() {
            return Err(BackupCutRecoveryCodecDenial::InvalidEncoding);
        }
        let (taken, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        Ok(taken)
    }

    fn take_u8(&mut self) -> Result<u8, BackupCutRecoveryCodecDenial> {
        Ok(self.take(1)?[0])
    }

    fn take_u32(&mut self) -> Result<u32, BackupCutRecoveryCodecDenial> {
        Ok(u32::from_le_bytes(self.take_array()?))
    }

    fn take_u64(&mut self) -> Result<u64, BackupCutRecoveryCodecDenial> {
        Ok(u64::from_le_bytes(self.take_array()?))
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], BackupCutRecoveryCodecDenial> {
        self.take(N)?
            .try_into()
            .map_err(|_| BackupCutRecoveryCodecDenial::InvalidEncoding)
    }

    fn take_bytes(&mut self) -> Result<&'a [u8], BackupCutRecoveryCodecDenial> {
        let count = self.take_u32()? as usize;
        self.take(count)
    }

    fn take_string(&mut self) -> Result<String, BackupCutRecoveryCodecDenial> {
        String::from_utf8(copy_bytes(self.take_bytes()?)?)
            .map_err(|_| BackupCutRecoveryCodecDenial::InvalidEncoding)
    }

    const fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_source_count_is_denied_before_source_allocation() {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(MAGIC);
        encoded.extend_from_slice(&[0; 32]);
        encoded.extend_from_slice(&0u64.to_le_bytes());
        for _ in 0..3 {
            encoded.extend_from_slice(&0u32.to_le_bytes());
        }
        encoded.extend_from_slice(&((MAX_RECOVERY_SOURCES as u32) + 1).to_le_bytes());
        let checksum = Sha256::digest(&encoded);
        encoded.extend_from_slice(&checksum);

        assert!(matches!(
            decode_backup_cut_recovery(&encoded),
            Err(BackupCutRecoveryCodecDenial::SizeLimitExceeded)
        ));
    }
}
