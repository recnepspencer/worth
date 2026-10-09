use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

/// Exact unpublished candidate frames reconstructed from a durable retirement
/// descriptor. This capability cannot name a selected root or payload file.
#[derive(Clone)]
pub(in crate::physical_runtime) struct RetirementCandidateRetryScope {
    frames: std::sync::Arc<std::collections::BTreeMap<RecordFrameCoordinate, [u8; 32]>>,
}

impl RetirementCandidateRetryScope {
    pub(in crate::physical_runtime::record_serving) fn admit(
        release: crate::physical_runtime::durability::RetirementReleaseProjection,
        selected_generation: u64,
        plan: &super::publication::PublicationPlan,
    ) -> Option<Self> {
        if selected_generation != release.source_generation()
            || plan.generation != release.candidate_generation()
            || plan.candidate
                != (RecordArtifactFile::CatalogCandidate {
                    publication: release.publication(),
                })
            || <[u8; 32]>::from(Sha256::digest(&plan.root_bytes)) != release.candidate_digest()
        {
            return None;
        }
        if plan.root
            != (RecordArtifactFile::RootManifest {
                generation: plan.generation,
            })
            || plan.previous_selector_candidate
                != (RecordArtifactFile::RootSelectorCandidate {
                    role: worth_store_physical_format::RootSelectorRole::Previous,
                    publication: release.publication(),
                })
            || plan.current_selector_candidate
                != (RecordArtifactFile::RootSelectorCandidate {
                    role: worth_store_physical_format::RootSelectorRole::Current,
                    publication: release.publication(),
                })
        {
            return None;
        }
        let mut frames = std::collections::BTreeMap::new();
        for (coordinate, bytes) in &plan.manifests {
            match coordinate.artifact() {
                RecordArtifactFile::FreeSpaceManifest { generation }
                | RecordArtifactFile::FreeSpaceMembershipBlock { generation, .. }
                | RecordArtifactFile::RootRoutingBlock { generation, .. }
                    if generation == plan.generation && coordinate.offset() == 0 => {}
                _ => return None,
            }
            frames.insert(*coordinate, Sha256::digest(bytes).into());
        }
        for (artifact, bytes) in [
            (plan.root, &plan.root_bytes),
            (plan.candidate, &plan.catalog_bytes),
            (
                plan.previous_selector_candidate,
                &plan.previous_selector_bytes,
            ),
            (
                plan.current_selector_candidate,
                &plan.current_selector_bytes,
            ),
        ] {
            frames.insert(
                RecordFrameCoordinate::new(artifact, 0, u32::try_from(bytes.len()).ok()?)?,
                Sha256::digest(bytes).into(),
            );
        }
        Some(Self {
            frames: std::sync::Arc::new(frames),
        })
    }
    pub(in crate::physical_runtime) fn admits(
        &self,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
    ) -> bool {
        self.frames
            .get(&coordinate)
            .is_some_and(|digest| *digest == <[u8; 32]>::from(Sha256::digest(bytes)))
    }
}
