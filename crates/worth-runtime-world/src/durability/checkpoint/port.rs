//! A weak binding port permits capture while the stopped World is still owned.
use super::{WorldCheckpointImage, WorldCheckpointReadSource, WorldCheckpointSubject};
use crate::durability::{
    WorldDurableMediumBinding, WorldDurableMediumDenial as Denial, WorldReadmissionDenial,
};
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionRequest, MapKernelContext};

#[derive(Clone, Debug)]
pub struct WorldCheckpointPort {
    binding: Weak<WorldDurableMediumBinding>,
}
impl WorldCheckpointPort {
    /// Actual selected/acknowledged Branches, Custody and Occurrences positions.
    /// Descriptive scheduling metadata; it proves no owner-state absence.
    pub fn acknowledged_positions(&self) -> Result<[u64; 3], Denial> {
        let binding = self.binding.upgrade().ok_or(Denial::Unavailable)?;
        let acknowledgement = binding
            .checkpoint_acknowledgement
            .try_lock()
            .map_err(|_| Denial::PublicationBusy)?;
        let coverage = &acknowledgement.coverage;
        Ok([
            coverage.branches.position,
            coverage.custody.position,
            coverage.occurrences.position,
        ])
    }
    pub(crate) fn new(binding: &Arc<WorldDurableMediumBinding>) -> Self {
        Self {
            binding: Arc::downgrade(binding),
        }
    }
    pub fn observe_program_activation_history(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<super::WorldProgramActivationHistory, WorldReadmissionDenial> {
        let binding = self
            .binding
            .upgrade()
            .ok_or(WorldReadmissionDenial::Medium(Denial::Unavailable))?;
        super::WorldProgramActivationHistory::capture(binding, request)
    }
    /// The restricted Home composition owns the only selection/publication
    /// journey. This operation only prepares original owner meaning.
    pub fn prepare_home_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedWorldCheckpoint, WorldReadmissionDenial> {
        let binding = self
            .binding
            .upgrade()
            .ok_or(WorldReadmissionDenial::Medium(Denial::Unavailable))?;
        request
            .consult()
            .map_err(|cause| WorldReadmissionDenial::Medium(Denial::Request(cause)))?;
        let source = binding
            .capture_checkpoint_source(request)
            .map_err(WorldReadmissionDenial::Medium)?;
        let (image, _) = WorldCheckpointImage::read_protected_source(
            source.as_ref(),
            binding.binding(),
            request,
            None,
        )?;
        let bytes = image
            .encode(request)
            .map_err(WorldReadmissionDenial::Medium)?;
        let subject =
            WorldCheckpointSubject::new(request, bytes, binding.binding().clone(), image.coverage)
                .map_err(WorldReadmissionDenial::Medium)?;
        let prepared = PreparedWorldCheckpoint {
            binding,
            source,
            subject,
        };
        prepared
            .validate_publication()
            .map_err(WorldReadmissionDenial::Medium)?;
        Ok(prepared)
    }
}

/// The original binding, protected W source, bytes and grants survive every
/// physical unknown or partial acknowledgement in the SAME Home attempt.
pub struct PreparedWorldCheckpoint {
    binding: Arc<WorldDurableMediumBinding>,
    source: Box<dyn WorldCheckpointReadSource>,
    subject: WorldCheckpointSubject,
}
impl std::fmt::Debug for PreparedWorldCheckpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedWorldCheckpoint")
            .field("subject", &self.subject)
            .finish_non_exhaustive()
    }
}
impl PreparedWorldCheckpoint {
    pub fn validate_previous_selected(
        &self,
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let before = WorldCheckpointImage::decode_selected(bytes, self.binding.binding(), request)?;
        super::acknowledgement::validate_successor(&before.coverage, self.subject.coverage())
            .map_err(WorldReadmissionDenial::Medium)
    }
    pub fn bytes(&self) -> &[u8] {
        self.subject.bytes()
    }
    pub fn publication_subject(&self) -> &WorldCheckpointSubject {
        &self.subject
    }
    pub fn validate_publication(&self) -> Result<(), Denial> {
        if self.subject.binding().owner() != self.binding.binding().owner()
            || self.subject.binding().program_revision()
                != self.binding.binding().program_revision()
        {
            return Err(Denial::StaleCheckpoint);
        }
        self.binding
            .checkpoint_acknowledgement
            .try_lock()
            .map_err(|_| Denial::PublicationBusy)?
            .validate(self.subject.coverage())
    }
    pub fn acknowledge_selected(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        context
            .poll()
            .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        let selected = self
            .source
            .observe_selection(&self.subject)?
            .ok_or(Denial::CheckpointSelectionRequired)?;
        self.binding
            .checkpoint_acknowledgement
            .try_lock()
            .map_err(|_| Denial::PublicationBusy)?
            .acknowledge(&self.subject, &selected)
    }
}
