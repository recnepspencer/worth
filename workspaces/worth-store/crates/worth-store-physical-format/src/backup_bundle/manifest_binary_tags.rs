use super::{BackupBundleArtifactFamily, BackupBundleArtifactFormat};

pub(super) const fn family_tag(family: BackupBundleArtifactFamily) -> u8 {
    match family {
        BackupBundleArtifactFamily::RootManifest => 1,
        BackupBundleArtifactFamily::CheckpointManifest => 2,
        BackupBundleArtifactFamily::WalSegment => 3,
        BackupBundleArtifactFamily::Page => 4,
        BackupBundleArtifactFamily::Extent => 5,
        BackupBundleArtifactFamily::Index => 6,
        BackupBundleArtifactFamily::BlobChunk => 7,
        BackupBundleArtifactFamily::SecondaryRoot => 8,
    }
}

pub(super) const fn family_from_tag(tag: u8) -> Option<BackupBundleArtifactFamily> {
    Some(match tag {
        1 => BackupBundleArtifactFamily::RootManifest,
        2 => BackupBundleArtifactFamily::CheckpointManifest,
        3 => BackupBundleArtifactFamily::WalSegment,
        4 => BackupBundleArtifactFamily::Page,
        5 => BackupBundleArtifactFamily::Extent,
        6 => BackupBundleArtifactFamily::Index,
        7 => BackupBundleArtifactFamily::BlobChunk,
        8 => BackupBundleArtifactFamily::SecondaryRoot,
        _ => return None,
    })
}

pub(super) const fn format_tag(format: BackupBundleArtifactFormat) -> u8 {
    match format {
        BackupBundleArtifactFormat::PhysicalRootManifestV1 => 1,
        BackupBundleArtifactFormat::RecoveryCheckpointManifestV1 => 2,
        BackupBundleArtifactFormat::WalSegmentV1 => 3,
        BackupBundleArtifactFormat::PhysicalDataPageV1 => 4,
        BackupBundleArtifactFormat::PhysicalExtentRecordV1 => 5,
        BackupBundleArtifactFormat::BTreeNodeV1 => 10,
        BackupBundleArtifactFormat::BlobChunkV1 => 8,
        BackupBundleArtifactFormat::PhysicalSecondaryRootManifestV1 => 9,
    }
}

pub(super) const fn format_from_tag(tag: u8) -> Option<BackupBundleArtifactFormat> {
    Some(match tag {
        1 => BackupBundleArtifactFormat::PhysicalRootManifestV1,
        2 => BackupBundleArtifactFormat::RecoveryCheckpointManifestV1,
        3 => BackupBundleArtifactFormat::WalSegmentV1,
        4 => BackupBundleArtifactFormat::PhysicalDataPageV1,
        5 => BackupBundleArtifactFormat::PhysicalExtentRecordV1,
        10 => BackupBundleArtifactFormat::BTreeNodeV1,
        8 => BackupBundleArtifactFormat::BlobChunkV1,
        9 => BackupBundleArtifactFormat::PhysicalSecondaryRootManifestV1,
        _ => return None,
    })
}
