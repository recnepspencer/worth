use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use worth_foundational::CanonicalDigestId;

use sha2::{Digest, Sha256};

use super::{FixtureScaleDeclaration, SyntheticFixtureAuthorityDenied};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterializedFixtureScaleEvidence {
    scale: FixtureScaleDeclaration,
    observed_store_bytes: u64,
    observed_blob_bytes: u64,
    observed_wal_bytes: u64,
    observed_damage_bytes: u64,
    sampled_read_count: u64,
    sampled_bytes_read: u64,
    maximum_resident_buffer_bytes: u64,
    evidence_identity: CanonicalDigestId,
}

impl MaterializedFixtureScaleEvidence {
    pub fn materialize(
        root: &Path,
        scale: FixtureScaleDeclaration,
    ) -> Result<Self, SyntheticFixtureAuthorityDenied> {
        std::fs::create_dir_all(root)
            .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
        let rows = [
            ("store.media", scale.declared_store_bytes(), 1_u8),
            ("blob.media", scale.blob_bytes(), 2_u8),
            ("wal.media", scale.wal_tail_bytes(), 3_u8),
            ("damage.media", scale.damaged_region_bytes(), 4_u8),
        ];
        let mut observed = [0_u64; 4];
        let mut sampled_read_count = 0_u64;
        let mut sampled_bytes_read = 0_u64;
        let mut digest = Sha256::new();
        digest.update(b"worth-store-materialized-fixture-scale-v1");
        for (index, (name, declared, tag)) in rows.into_iter().enumerate() {
            let extent = materialize_sparse_extent(&root.join(name), declared, tag)?;
            observed[index] = extent.observed_length;
            sampled_read_count = sampled_read_count.saturating_add(extent.sampled_read_count);
            sampled_bytes_read = sampled_bytes_read.saturating_add(extent.sampled_bytes_read);
            digest.update([tag]);
            digest.update(observed[index].to_be_bytes());
            digest.update(extent.sampled_read_count.to_be_bytes());
            digest.update(extent.sampled_bytes_read.to_be_bytes());
            digest.update(extent.sample_digest.bytes());
            digest.update(boundary_sentinel(tag, declared));
        }
        Ok(Self {
            scale,
            observed_store_bytes: observed[0],
            observed_blob_bytes: observed[1],
            observed_wal_bytes: observed[2],
            observed_damage_bytes: observed[3],
            sampled_read_count,
            sampled_bytes_read,
            maximum_resident_buffer_bytes: 32,
            evidence_identity: CanonicalDigestId::new(digest.finalize().into()),
        })
    }

    pub const fn scale(self) -> FixtureScaleDeclaration {
        self.scale
    }

    pub const fn evidence_identity(self) -> [u8; 32] {
        *self.evidence_identity.bytes()
    }

    pub const fn matches_declared_scale(self) -> bool {
        self.observed_store_bytes == self.scale.declared_store_bytes()
            && self.observed_blob_bytes == self.scale.blob_bytes()
            && self.observed_wal_bytes == self.scale.wal_tail_bytes()
            && self.observed_damage_bytes == self.scale.damaged_region_bytes()
    }

    pub const fn traversed_declared_media(self) -> bool {
        self.matches_declared_scale()
            && self.sampled_read_count >= 4
            && self.sampled_bytes_read >= self.sampled_read_count
            && self.maximum_resident_buffer_bytes == 32
    }

    pub const fn sampled_read_count(self) -> u64 {
        self.sampled_read_count
    }

    pub const fn maximum_resident_buffer_bytes(self) -> u64 {
        self.maximum_resident_buffer_bytes
    }
}

struct MaterializedExtentObservation {
    observed_length: u64,
    sampled_read_count: u64,
    sampled_bytes_read: u64,
    sample_digest: CanonicalDigestId,
}

fn materialize_sparse_extent(
    path: &Path,
    length: u64,
    tag: u8,
) -> Result<MaterializedExtentObservation, SyntheticFixtureAuthorityDenied> {
    if length < 64 {
        return Err(SyntheticFixtureAuthorityDenied::ScaleMediaMismatch);
    }
    let sentinel = boundary_sentinel(tag, length);
    prepare_sparse_file(path)?;
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.set_len(length)
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.write_all(&sentinel)
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.seek(SeekFrom::Start(length - sentinel.len() as u64))
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.write_all(&sentinel)
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.sync_all()
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    let observed = file
        .metadata()
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?
        .len();
    let mut actual = [0_u8; 32];
    file.seek(SeekFrom::Start(length - 32))
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    file.read_exact(&mut actual)
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    if observed != length || actual != sentinel {
        return Err(SyntheticFixtureAuthorityDenied::ScaleMediaMismatch);
    }
    let (sampled_read_count, sampled_bytes_read, sample_digest) = sample_extent(&mut file, length)?;
    Ok(MaterializedExtentObservation {
        observed_length: observed,
        sampled_read_count,
        sampled_bytes_read,
        sample_digest: CanonicalDigestId::new(sample_digest),
    })
}

fn sample_extent(
    file: &mut std::fs::File,
    length: u64,
) -> Result<(u64, u64, [u8; 32]), SyntheticFixtureAuthorityDenied> {
    const MAX_SAMPLE_COUNT: u64 = 4_096;
    const BUFFER_BYTES: usize = 32;

    let stride = length.div_ceil(MAX_SAMPLE_COUNT).max(BUFFER_BYTES as u64);
    let mut offset = 0_u64;
    let mut reads = 0_u64;
    let mut bytes_read = 0_u64;
    let mut digest = Sha256::new();
    digest.update(b"worth-store-materialized-scale-sampled-walk-v1");
    while offset < length {
        let remaining = length - offset;
        let width = remaining.min(BUFFER_BYTES as u64) as usize;
        let mut sample = [0_u8; BUFFER_BYTES];
        file.seek(SeekFrom::Start(offset))
            .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
        file.read_exact(&mut sample[..width])
            .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
        digest.update(offset.to_be_bytes());
        digest.update((width as u64).to_be_bytes());
        digest.update(&sample[..width]);
        reads = reads.saturating_add(1);
        bytes_read = bytes_read.saturating_add(width as u64);
        offset = offset.saturating_add(stride);
    }
    Ok((reads, bytes_read, digest.finalize().into()))
}

fn prepare_sparse_file(path: &Path) -> Result<(), SyntheticFixtureAuthorityDenied> {
    std::fs::File::create(path)
        .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
    #[cfg(windows)]
    {
        let output = std::process::Command::new("fsutil")
            .args(["sparse", "setflag"])
            .arg(path)
            .output()
            .map_err(|_| SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable)?;
        if !output.status.success() {
            return Err(SyntheticFixtureAuthorityDenied::ScaleMediaUnavailable);
        }
    }
    Ok(())
}

fn boundary_sentinel(tag: u8, length: u64) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-materialized-scale-boundary-v1");
    digest.update([tag]);
    digest.update(length.to_be_bytes());
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::profile::FixtureRecoveryScale;
    use crate::{FixtureActivityScale, FixtureStorageScale, LargeStoreFixtureProfile};

    #[test]
    fn materialized_scale_is_derived_from_observed_media_lengths() {
        let root = tempfile::tempdir().unwrap();
        let scale = FixtureScaleDeclaration::new(
            LargeStoreFixtureProfile::StoreLargerThanMemory,
            FixtureStorageScale::new(4096, 128),
            FixtureActivityScale::new(1, 1, 128, 128),
            FixtureRecoveryScale::new(2048, 1024, 512),
            None,
        );

        let evidence = MaterializedFixtureScaleEvidence::materialize(root.path(), scale).unwrap();

        assert!(evidence.matches_declared_scale());
        assert!(evidence.traversed_declared_media());
        assert!(evidence.sampled_read_count() >= 4);
        assert_ne!(evidence.evidence_identity(), [0; 32]);
        assert_eq!(
            std::fs::metadata(root.path().join("store.media"))
                .unwrap()
                .len(),
            4096
        );
        assert_eq!(
            std::fs::metadata(root.path().join("blob.media"))
                .unwrap()
                .len(),
            2048
        );
    }
}
