use worth_store_physical_backend::ArtifactTreeFailure;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, RecordArtifactFile,
};

use super::super::residency::serving_artifacts::ServingRecordArtifacts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecordPublicationResidueObservation {
    staging_catalog_candidate: bool,
    successor_root: bool,
    successor_routing_block: bool,
    successor_segment_membership_block: bool,
    successor_free_space_membership_block: bool,
    successor_free_space: bool,
    next_segment_data: bool,
    reusable_segment_data: bool,
    next_arena: bool,
}

impl RecordPublicationResidueObservation {
    /// These observed names can be explained by a sealed free-map release.
    /// This is not an exhaustive inventory or an admission of their bytes.
    pub(in crate::physical_runtime) const fn permits_retirement_candidate_reconstruction(
        self,
    ) -> bool {
        !self.successor_routing_block
            && !self.successor_segment_membership_block
            && !self.next_segment_data
            && !self.reusable_segment_data
            && !self.next_arena
    }
    pub const fn is_empty(self) -> bool {
        !(self.staging_catalog_candidate
            || self.successor_root
            || self.successor_routing_block
            || self.successor_segment_membership_block
            || self.successor_free_space_membership_block
            || self.successor_free_space
            || self.next_segment_data
            || self.reusable_segment_data
            || self.next_arena)
    }

    pub const fn staging_catalog_candidate(self) -> bool {
        self.staging_catalog_candidate
    }
    pub const fn successor_root(self) -> bool {
        self.successor_root
    }
    pub const fn successor_routing_block(self) -> bool {
        self.successor_routing_block
    }
    pub const fn successor_free_space(self) -> bool {
        self.successor_free_space
    }
    pub const fn successor_membership_blocks(self) -> bool {
        self.successor_segment_membership_block || self.successor_free_space_membership_block
    }
    pub const fn next_segment_artifacts(self) -> bool {
        self.next_segment_data
    }
    pub const fn reusable_segment_artifacts(self) -> bool {
        self.reusable_segment_data
    }
    pub const fn next_arena_artifact(self) -> bool {
        self.next_arena
    }

    /// A new arena name alone may be explained by one WAL-held, unpublished
    /// copy destination. Other candidate names still require inspection.
    pub(in crate::physical_runtime) const fn only_next_arena_artifact(self) -> bool {
        let mut other = self;
        other.next_arena = false;
        self.next_arena && other.is_empty()
    }
}

pub(in crate::physical_runtime::record_serving) fn observe_publication_residue(
    artifacts: &ServingRecordArtifacts<'_>,
    current_root: &DurablePhysicalRootManifest,
    free_space: &DurableFreeSpaceManifestHeader,
    staging_catalog_candidate: bool,
) -> Result<RecordPublicationResidueObservation, ArtifactTreeFailure> {
    let Some(successor_generation) = current_root.generation().checked_add(1) else {
        return Ok(RecordPublicationResidueObservation {
            staging_catalog_candidate,
            ..RecordPublicationResidueObservation::default()
        });
    };
    let mut observation = RecordPublicationResidueObservation {
        staging_catalog_candidate,
        successor_root: exists(
            artifacts,
            RecordArtifactFile::RootManifest {
                generation: successor_generation,
            },
        )?,
        successor_routing_block: exists(
            artifacts,
            RecordArtifactFile::RootRoutingBlock {
                generation: successor_generation,
                block: current_root.next_block(),
            },
        )?,
        successor_segment_membership_block: exists(
            artifacts,
            RecordArtifactFile::SegmentMembershipBlock {
                generation: successor_generation,
                block: current_root.next_segment_block(),
            },
        )?,
        successor_free_space_membership_block: exists(
            artifacts,
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: successor_generation,
                block: free_space.next_block(),
            },
        )?,
        successor_free_space: exists(
            artifacts,
            RecordArtifactFile::FreeSpaceManifest {
                generation: successor_generation,
            },
        )?,
        next_segment_data: exists(
            artifacts,
            RecordArtifactFile::Segment {
                segment: free_space.next_segment(),
                generation: 1,
            },
        )?,
        next_arena: exists(
            artifacts,
            RecordArtifactFile::ExtentArena {
                arena: free_space.next_arena(),
            },
        )?,
        ..RecordPublicationResidueObservation::default()
    };
    if let Some(segment) = current_root.last_inline_segment() {
        if let Some(generation) = segment.generation().get().checked_add(1) {
            observation.reusable_segment_data = exists(
                artifacts,
                RecordArtifactFile::Segment {
                    segment: segment.segment_id().get(),
                    generation,
                },
            )?;
        }
    }
    Ok(observation)
}

fn exists(
    artifacts: &ServingRecordArtifacts<'_>,
    artifact: RecordArtifactFile,
) -> Result<bool, ArtifactTreeFailure> {
    artifacts.file_exists(artifact)
}

#[cfg(test)]
mod tests {
    use super::RecordPublicationResidueObservation as Residue;

    #[test]
    fn copy_residue_requires_only_the_next_arena_name() {
        let exact = Residue {
            next_arena: true,
            ..Residue::default()
        };
        assert!(exact.only_next_arena_artifact());
        assert!(!Residue::default().only_next_arena_artifact());
        assert!(!Residue {
            successor_root: true,
            ..exact
        }
        .only_next_arena_artifact());
        assert!(!Residue {
            staging_catalog_candidate: true,
            ..exact
        }
        .only_next_arena_artifact());
        assert!(!Residue {
            successor_free_space: true,
            ..exact
        }
        .only_next_arena_artifact());
        assert!(!Residue {
            next_segment_data: true,
            ..exact
        }
        .only_next_arena_artifact());
    }
}
