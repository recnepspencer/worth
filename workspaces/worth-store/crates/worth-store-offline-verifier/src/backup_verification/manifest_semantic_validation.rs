use worth_store_physical_format::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleManifest,
};

use super::BackupVerificationDefect;

const REQUIRED_FAMILIES: &[BackupBundleArtifactFamily] = &[
    BackupBundleArtifactFamily::RootManifest,
    BackupBundleArtifactFamily::CheckpointManifest,
    BackupBundleArtifactFamily::WalSegment,
    BackupBundleArtifactFamily::Page,
    BackupBundleArtifactFamily::Extent,
    BackupBundleArtifactFamily::Index,
    BackupBundleArtifactFamily::BlobChunk,
];

pub(super) fn validate_manifest(
    manifest: &BackupBundleManifest,
    defects: &mut Vec<BackupVerificationDefect>,
) -> Result<u64, ()> {
    let rows = manifest.artifacts();
    let mut wal_intervals = Vec::new();
    wal_intervals
        .try_reserve_exact(rows.len())
        .map_err(|_| ())?;
    for row in rows {
        if let BackupBundleArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } = row.coverage()
        {
            wal_intervals.push((*start_lsn, *end_exclusive_lsn));
        }
        if !row.coverage().matches_family(row.family()) {
            defects.push(BackupVerificationDefect::CoverageFamilyMismatch {
                output_name: row.output_name().to_owned(),
            });
        }
    }
    validate_required_families(rows, defects);
    validate_root(manifest, defects);
    validate_checkpoint(manifest, defects);
    validate_wal(manifest, &mut wal_intervals, defects);
    u64::try_from(wal_intervals.capacity())
        .ok()
        .and_then(|capacity| capacity.checked_mul(std::mem::size_of::<(u64, u64)>() as u64))
        .ok_or(())
}

fn validate_required_families(
    rows: &[worth_store_physical_format::BackupBundleArtifactManifestRow],
    defects: &mut Vec<BackupVerificationDefect>,
) {
    for family in REQUIRED_FAMILIES {
        if !rows.iter().any(|row| row.family() == *family) {
            defects.push(BackupVerificationDefect::MissingArtifactFamily(*family));
        }
    }
}

fn validate_root(manifest: &BackupBundleManifest, defects: &mut Vec<BackupVerificationDefect>) {
    let mut roots = manifest
        .artifacts()
        .iter()
        .filter(|row| row.family() == BackupBundleArtifactFamily::RootManifest);
    let root = roots.next();
    let exactly_one = root.is_some() && roots.next().is_none();
    if !exactly_one || root.is_some_and(|row| row.generation() != manifest.root_generation()) {
        defects.push(BackupVerificationDefect::RootGenerationMismatch);
    }
    if !exactly_one
        || !root.is_some_and(|row| {
            matches!(
                row.coverage(),
                BackupBundleArtifactCoverage::RootManifest { root_generation }
                    if *root_generation == manifest.root_generation()
            )
        })
    {
        defects.push(BackupVerificationDefect::RootCoverageMismatch);
    }
}

fn validate_checkpoint(
    manifest: &BackupBundleManifest,
    defects: &mut Vec<BackupVerificationDefect>,
) {
    let mut checkpoints = manifest
        .artifacts()
        .iter()
        .filter(|row| row.family() == BackupBundleArtifactFamily::CheckpointManifest);
    let checkpoint = checkpoints.next();
    let exactly_one = checkpoint.is_some() && checkpoints.next().is_none();
    if !exactly_one
        || checkpoint.is_some_and(|row| row.generation() != manifest.manifest_generation())
    {
        defects.push(BackupVerificationDefect::CheckpointGenerationMismatch);
    }
    if !exactly_one
        || !checkpoint.is_some_and(|row| {
            matches!(
                row.coverage(),
                BackupBundleArtifactCoverage::CheckpointManifest {
                    checkpoint_identity,
                    manifest_generation,
                    durable_checkpoint_lsn,
                    frontier_digest,
                    ..
                } if checkpoint_identity == manifest.checkpoint_identity()
                    && *manifest_generation == manifest.manifest_generation()
                    && *durable_checkpoint_lsn == manifest.durable_checkpoint_lsn()
                    && *frontier_digest != [0; 32]
            )
        })
    {
        defects.push(BackupVerificationDefect::CheckpointCoverageMismatch);
    }
}

fn validate_wal(
    manifest: &BackupBundleManifest,
    intervals: &mut [(u64, u64)],
    defects: &mut Vec<BackupVerificationDefect>,
) {
    intervals.sort_unstable();
    let expected = manifest.wal_half_open_interval();
    let exact = intervals.first().map(|range| range.0) == Some(expected.0)
        && intervals.last().map(|range| range.1) == Some(expected.1)
        && !intervals.windows(2).any(|pair| pair[0].1 != pair[1].0);
    if !exact {
        defects.push(BackupVerificationDefect::WalCoverageGapOrOverlap);
    }
}
