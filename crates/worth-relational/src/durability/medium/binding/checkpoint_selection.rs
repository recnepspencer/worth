//! Only the attached Native binding mints and consumes selection authority.
use super::RelationalDurableMediumBinding;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::durability::medium::{
    CheckpointSelectionIdentity, CheckpointSelectionObservation, OpaqueCapturedCheckpointSubject,
    RelationalCheckpointAcknowledgementDenial as Denial,
};
use std::sync::{Mutex, Weak};
use worth_proof::{AuthorityMarker, DurablyCommitted};
mod region_displacements;
use region_displacements::RegionDisplacements;

#[derive(Debug)]
pub(crate) struct CheckpointPublication {
    subject: OpaqueCapturedCheckpointSubject,
    binding: Weak<RelationalDurableMediumBinding>,
    phase: Mutex<AcknowledgementPhase>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug)]
enum AcknowledgementPhase {
    Captured {
        capture: Option<RelationalNativeCheckpoint>,
        retired:
            Option<crate::durability::checkpoint_history::publication::RetiredCanonicalPlacements>,
        regions: Option<RegionDisplacements>,
    },
    Acknowledging(AcknowledgingCheckpoint),
    Acknowledged {
        _completed: AcknowledgingCheckpoint,
    },
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
    admission: Option<crate::durability::data::CheckpointDirectoryAdmission>,
    directory: Option<std::sync::Arc<crate::durability::data::RelationalCheckpointDirectory>>,
    next_canonical: usize,
    retired: crate::durability::checkpoint_history::publication::RetiredCanonicalPlacements,
    next_region: usize,
    regions: RegionDisplacements,
}

impl CheckpointPublication {
    pub(crate) fn new(
        subject: OpaqueCapturedCheckpointSubject,
        binding: Weak<RelationalDurableMediumBinding>,
        capture: RelationalNativeCheckpoint,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::data::RelationalNativeCheckpointCaptureDenial> {
        use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Capture;
        let size = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| {
                Capture::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(size as u64)
            .map_err(|cause| Capture::Request(cause.into()))?;
        let retired=crate::durability::checkpoint_history::publication::RetiredCanonicalPlacements::prepare(request,capture.canonical_publication().map_or(0,|publication|publication.len()))?;
        let regions = RegionDisplacements::prepare(request, capture.region_publications().len())?;
        Ok(Self {
            subject,
            binding,
            phase: Mutex::new(AcknowledgementPhase::Captured {
                capture: Some(capture),
                retired: Some(retired),
                regions: Some(regions),
            }),
            _memory: memory,
        })
    }
    pub(crate) fn subject(&self) -> &OpaqueCapturedCheckpointSubject {
        &self.subject
    }

    pub(crate) fn advance(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        let mut phase = match self.phase.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        if let AcknowledgementPhase::Captured {
            capture,
            retired,
            regions,
        } = &mut *phase
        {
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
                    admission: None,
                    directory: None,
                    next_canonical: 0,
                    retired: retired
                        .take()
                        .expect("captured phase retains exact original placement slots"),
                    next_region: 0,
                    regions: regions
                        .take()
                        .expect("captured phase retains every region metadata slot"),
                })
            });
        }
        if let AcknowledgementPhase::Acknowledging(acknowledging) = &mut *phase {
            acknowledging.advance(execution, context)?;
            acknowledging.capture.release_component_holds();
            let prior = std::mem::replace(
                &mut *phase,
                AcknowledgementPhase::Captured {
                    capture: None,
                    retired: None,
                    regions: None,
                },
            );
            let AcknowledgementPhase::Acknowledging(completed) = prior else {
                unreachable!("exclusive acknowledgement owns the same phase")
            };
            *phase = AcknowledgementPhase::Acknowledged {
                _completed: completed,
            };
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        if self.directory.is_none() {
            execution.check_live().map_err(Denial::Stopped)?;
            if let Some(publication) = self.capture.manifest_publication() {
                let admission = self
                    .admission
                    .get_or_insert_with(|| publication.begin_selected_directory_admission());
                let base = publication
                    .capture_base()
                    .map(|slot| slot.snapshot().ok_or(Denial::CaptureNotBound))
                    .transpose()?;
                let mut directory = admission.advance(execution)?;
                if let Some(base) = base {
                    if directory.manifest.covered_position != base.source().covered()
                        || directory.manifest.coverage.up_to_branch_transition
                            != base.source().covered_transitions()
                    {
                        return Err(Denial::CaptureNotBound);
                    }
                    directory.admission_image = Some(base.image().clone());
                }
                self.directory = Some(std::sync::Arc::new(directory));
            }
        }
        if self.capture.manifest_publication().is_some()
            && let Some(publication) = self.capture.canonical_publication()
        {
            let directory = self.directory.as_ref().ok_or(Denial::CaptureNotBound)?;
            publication.advance(
                &mut self.next_canonical,
                &mut self.retired,
                directory,
                execution,
                context,
            )?;
        }
        while let Some(publication) = self.capture.region_publications().get(self.next_region) {
            if context.has_accounting() {
                context.checkpoint(1).map_err(Denial::Stopped)?;
            } else {
                context.poll().map_err(Denial::Stopped)?;
            }
            let slot = self.regions.slot(self.next_region);
            if self
                .capture
                .retains_reconstructed_regions()
                .map_err(|()| Denial::Busy)?
            {
                slot.controlled = publication.try_acknowledge_for_request(execution).map_err(|cause| match cause {
                    crate::storage::residency::RegionCheckpointAcknowledgementDenial::Busy=>Denial::Busy,
                    crate::storage::residency::RegionCheckpointAcknowledgementDenial::Stopped(cause)=>Denial::Recovery(cause),
                })?;
            } else {
                slot.ordinary = publication.try_acknowledge().map_err(|()| Denial::Busy)?;
            }
            self.next_region += 1;
        }
        if !self.directory_applied {
            context.poll().map_err(Denial::Stopped)?;
            if let Some(publication) = self.capture.manifest_publication() {
                let prior = publication.acknowledge_directory(
                    self.directory
                        .as_ref()
                        .ok_or(Denial::CaptureNotBound)?
                        .clone(),
                )?;
                self.retired.displaced_selection(prior);
            }
            self.directory_applied = true;
        }
        self.retired.advance(
            execution,
            context,
            self.directory
                .as_ref()
                .map(|directory| directory.source.as_ref()),
        )?;
        if let Some(publication) = self.capture.manifest_publication() {
            publication.complete_acknowledgement()?;
        }
        Ok(())
    }
}
