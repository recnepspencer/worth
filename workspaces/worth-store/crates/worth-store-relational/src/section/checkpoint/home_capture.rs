//! An unframed genuine Home capture becomes one final admission section.
use super::complete::{
    PreparedCompleteRelationalSection, RelationalSectionCompleteCheckpointPreparation,
};
use super::publication::SectionPublication;
use super::selection_custody::SectionSelectionCustody;
use super::{RelationalSectionCheckpointDenial as Denial, RelationalStoreSection};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeIdentity;
use worth_relational::facade::durability::{
    CompleteRelationalNativeCheckpoint, RelationalCheckpointAdmissionArtifact,
};

/// Original Native capture, its sealed image, and the SAME live selection fence.
/// This phase has no serialized predecessor and cannot be selected or ACKed.
pub struct CapturedHomeRelationalSection {
    complete: CompleteRelationalNativeCheckpoint,
    custody: SectionSelectionCustody,
    image: RelationalCheckpointAdmissionArtifact,
}

impl RelationalSectionCompleteCheckpointPreparation {
    pub fn prepare_home(
        &self,
        home: HomeIdentity,
        world_through: [u64; 3],
        world_digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedHomeRelationalSection, Denial> {
        let (complete, source, selection) = self.capture_native(request)?;
        let mut custody =
            SectionSelectionCustody::prepare(&source, complete.checkpoint(), request)?;
        custody.selection = Some(selection);
        CapturedHomeRelationalSection::from_capture(
            complete,
            custody,
            home,
            world_through,
            world_digest,
            request,
        )
    }
}

impl RelationalStoreSection {
    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn capture_complete_home_checkpoint(
        &self,
        complete: CompleteRelationalNativeCheckpoint,
        home: HomeIdentity,
        world_through: [u64; 3],
        world_digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedHomeRelationalSection, Denial> {
        let mut custody =
            SectionSelectionCustody::prepare(&self.cold_source, complete.checkpoint(), request)?;
        custody.selection = Some(self.checkpoint_selection.clone());
        CapturedHomeRelationalSection::from_capture(
            complete,
            custody,
            home,
            world_through,
            world_digest,
            request,
        )
    }
}

impl CapturedHomeRelationalSection {
    fn from_capture(
        complete: CompleteRelationalNativeCheckpoint,
        custody: SectionSelectionCustody,
        home: HomeIdentity,
        world_through: [u64; 3],
        world_digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let image = complete
            .admission_image()
            .seal_home(home, world_through, world_digest, request)
            .map_err(Denial::AdmissionImage)?;
        Ok(Self {
            complete,
            custody,
            image,
        })
    }
    pub fn admission_image(&self) -> &RelationalCheckpointAdmissionArtifact {
        &self.image
    }
    /// Consume the actual original capture; the caller supplies its SAME World companion.
    pub fn bind_home_admission_section(
        self,
        world_companion: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedCompleteRelationalSection, Denial> {
        let (bytes, memory) = super::super::admission_section::encode(
            self.complete.source_artifact(),
            &self.image,
            world_companion,
            request,
        )?;
        let publication = SectionPublication::from_custody(
            self.custody,
            Some(self.complete.initial_predecessor()),
            bytes,
            memory,
        );
        Ok(PreparedCompleteRelationalSection {
            complete: self.complete,
            publication,
        })
    }
}
