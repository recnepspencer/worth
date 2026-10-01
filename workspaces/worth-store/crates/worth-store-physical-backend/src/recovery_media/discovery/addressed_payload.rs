use worth_store_physical_format::{ExtentArenaRange, RecordArtifactFile};

use super::{
    BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, RecoveryDiscoveryFailure,
};

impl BoundedRecoveryFilesystemDiscovery {
    pub fn read_release_custody_head_block(
        &mut self,
        generation: u64,
        block: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::ReleaseCustodyHeadBlock { generation, block },
            byte_limit,
        )
    }

    pub fn read_free_space_manifest(
        &mut self,
        generation: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::FreeSpaceManifest { generation },
            byte_limit,
        )
    }

    pub fn read_free_space_membership_block(
        &mut self,
        generation: u64,
        block: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::FreeSpaceMembershipBlock { generation, block },
            byte_limit,
        )
    }

    pub fn read_segment_manifest(
        &mut self,
        segment: u64,
        generation: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::SegmentManifest {
                segment,
                generation,
            },
            byte_limit,
        )
    }

    pub fn read_segment(
        &mut self,
        segment: u64,
        generation: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::Segment {
                segment,
                generation,
            },
            byte_limit,
        )
    }

    pub fn read_segment_membership_block(
        &mut self,
        generation: u64,
        block: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::SegmentMembershipBlock { generation, block },
            byte_limit,
        )
    }

    pub fn read_segment_range(
        &mut self,
        segment: u64,
        generation: u64,
        offset: u64,
        length: u32,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed_range(
            RecordArtifactFile::Segment {
                segment,
                generation,
            },
            offset,
            length,
            byte_limit,
        )
    }

    pub fn read_extent_manifest(
        &mut self,
        range: ExtentArenaRange,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_extent_range(range, 0, 104, byte_limit)
    }

    pub fn read_extent_range(
        &mut self,
        range: ExtentArenaRange,
        offset: u64,
        length: u32,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let artifact = RecordArtifactFile::ExtentArena {
            arena: range.arena().get(),
        };
        let context = super::RecoveryDiscoveryArtifact::Record(artifact);
        let end = offset
            .checked_add(u64::from(length))
            .filter(|end| *end <= range.length())
            .ok_or_else(|| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        let absolute = range
            .offset()
            .checked_add(end - u64::from(length))
            .ok_or_else(|| RecoveryDiscoveryFailure::invalid(context))?;
        self.read_addressed_range(artifact, absolute, length, byte_limit)
    }
}
