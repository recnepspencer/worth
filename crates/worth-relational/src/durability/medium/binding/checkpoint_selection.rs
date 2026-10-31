//! Only the attached Native binding mints and consumes selection authority.
use super::RelationalDurableMediumBinding;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::durability::medium::{
    CheckpointSelectionIdentity, CheckpointSelectionObservation, OpaqueCapturedCheckpointSubject,
    RelationalCheckpointAcknowledgementDenial as Denial,
};
use std::sync::{Mutex, Weak};
use worth_proof::{AuthorityMarker, DurablyCommitted};

#[derive(Debug)]
pub(crate) struct CheckpointPublication {
    subject: OpaqueCapturedCheckpointSubject,
    binding: Weak<RelationalDurableMediumBinding>,
    phase: Mutex<AcknowledgementPhase>,
}

#[derive(Debug)]
enum AcknowledgementPhase {
    Captured(Option<RelationalNativeCheckpoint>),
    Acknowledging(AcknowledgingCheckpoint),
    Acknowledged,
}

/// Neither its payload nor its constructor crosses Native's binding boundary.
struct ActualCheckpointSelection {
    capture: RelationalNativeCheckpoint,
    selection: CheckpointSelectionIdentity,
}
impl AuthorityMarker for ActualCheckpointSelection {}

#[derive(Debug)]
struct AcknowledgingCheckpoint {
    capture: RelationalNativeCheckpoint,
    _selection: CheckpointSelectionIdentity,
    directory_applied: bool,
    next_region: usize,
}

impl CheckpointPublication {
    pub(crate) fn new(
        subject: OpaqueCapturedCheckpointSubject,
        binding: Weak<RelationalDurableMediumBinding>,
        capture: RelationalNativeCheckpoint,
    ) -> Self {
        Self {
            subject,
            binding,
            phase: Mutex::new(AcknowledgementPhase::Captured(Some(capture))),
        }
    }
    pub(crate) fn subject(&self) -> &OpaqueCapturedCheckpointSubject {
        &self.subject
    }

    pub(crate) fn advance(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        let mut phase = self.phase.lock().unwrap_or_else(|error| error.into_inner());
        if let AcknowledgementPhase::Captured(capture) = &mut *phase {
            // No Home driver lock is held here. The medium's weak lookup reads
            // the exact pending receipt, never this acknowledgement mutex.
            let binding = self.binding.upgrade().ok_or(Denial::CaptureNotBound)?;
            let selection = binding.observe_checkpoint_selection(&self.subject)?;
            let proof = DurablyCommitted::from_owner(ActualCheckpointSelection {
                capture: capture
                    .take()
                    .expect("captured phase owns original capture"),
                selection,
            });
            // Install ownership before polling or performing any fallible work.
            *phase = proof.consume(|actual| {
                AcknowledgementPhase::Acknowledging(AcknowledgingCheckpoint {
                    capture: actual.capture,
                    _selection: actual.selection,
                    directory_applied: false,
                    next_region: 0,
                })
            });
        }
        if let AcknowledgementPhase::Acknowledging(acknowledging) = &mut *phase {
            acknowledging.advance(context)?;
            acknowledging.capture.release_component_holds();
            *phase = AcknowledgementPhase::Acknowledged;
        }
        Ok(())
    }
}

impl RelationalDurableMediumBinding {
    fn observe_checkpoint_selection(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Result<CheckpointSelectionIdentity, Denial> {
        let state = self.state();
        let medium = state.medium.as_ref().ok_or(Denial::CaptureNotBound)?;
        match medium.checkpoint_selection(subject) {
            CheckpointSelectionObservation::Selected(selection) => Ok(selection),
            CheckpointSelectionObservation::NotSelected
            | CheckpointSelectionObservation::Unknown => Err(Denial::SelectionNotKnown),
        }
    }
}

impl AcknowledgingCheckpoint {
    fn advance(
        &mut self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        if !self.directory_applied {
            context.poll().map_err(Denial::Stopped)?;
            if let Some(publication) = self.capture.manifest_publication() {
                publication.acknowledge().map_err(Denial::Storage)?;
            }
            self.directory_applied = true;
        }
        while let Some(publication) = self.capture.region_publications().get(self.next_region) {
            if context.has_accounting() {
                context.checkpoint(1).map_err(Denial::Stopped)?;
            } else {
                context.poll().map_err(Denial::Stopped)?;
            }
            publication.acknowledge();
            self.next_region += 1;
        }
        Ok(())
    }
}
