//! the actual lowering of one original Signal publication.
use super::{BridgeDurableDefinitionPublicationDenial as D, BridgeInstalledConditionalLowering};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{SignalBranchAdvanceIdentity, SignalDefinitionProjectionOutcome};
use worth_signal::facade::durability::{SignalDurableAdvanceOutput, SignalRecordedSourcePoint};

pub(super) struct InstalledDefinition {
    pub(super) original: SignalBranchAdvanceIdentity,
    pub(super) point: SignalRecordedSourcePoint,
    pub(super) lowering: Arc<BridgeInstalledConditionalLowering>,
}

pub(super) struct DefinitionCompletionSlot {
    pub(super) installed: Mutex<Option<InstalledDefinition>>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl DefinitionCompletionSlot {
    pub(super) fn reserve(execution: &ExecutionRequest<'_, '_>) -> Result<Arc<Self>, D> {
        use super::BridgeAuthoredSectionDenial as Section;
        let bytes = crate::conditional_execution::retention::arc_charge::<Self>()
            .map_err(|_| D::Section(Box::new(Section::SizeLimit)))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| D::Section(Box::new(Section::Request(cause))))?;
        Ok(Arc::new(Self {
            installed: Mutex::new(None),
            _memory: memory,
        }))
    }
}

/// Runtime completion is derived and cannot be restored from an executable snapshot.
/// Construction is private to the same prepared Bridge/Signal original.
pub struct BridgeDurableDefinitionCompletion {
    slot: Arc<DefinitionCompletionSlot>,
}

#[derive(Debug)]
pub enum BridgeDurableDefinitionCompletionDenial {
    ForeignOriginal {
        expected: SignalBranchAdvanceIdentity,
        observed: SignalBranchAdvanceIdentity,
    },
    ProjectionNotInstalled(SignalDefinitionProjectionOutcome),
    InstallationPending {
        original: SignalBranchAdvanceIdentity,
    },
    SourceMismatch {
        original: SignalBranchAdvanceIdentity,
    },
}

impl BridgeDurableDefinitionCompletion {
    pub(super) fn new(slot: Arc<DefinitionCompletionSlot>) -> Self {
        Self { slot }
    }

    pub fn installed_lowering(
        &self,
        output: &SignalDurableAdvanceOutput,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeDurableDefinitionCompletionDenial>
    {
        use BridgeDurableDefinitionCompletionDenial as Denial;
        let observed = output.artifact().identity();
        match output.projection_outcome() {
            SignalDefinitionProjectionOutcome::Installed => {}
            outcome @ (SignalDefinitionProjectionOutcome::NotRequired
            | SignalDefinitionProjectionOutcome::UnwoundAfterCommit) => {
                return Err(Denial::ProjectionNotInstalled(outcome));
            }
        }
        let slot = self
            .slot
            .installed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let installed = slot
            .as_ref()
            .ok_or(Denial::InstallationPending { original: observed })?;
        if installed.original != observed {
            return Err(Denial::ForeignOriginal {
                expected: installed.original,
                observed,
            });
        }
        if &installed.point != output.branch().source_point() {
            return Err(Denial::SourceMismatch { original: observed });
        }
        Ok(Arc::clone(&installed.lowering))
    }
}

impl std::fmt::Debug for BridgeDurableDefinitionCompletion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeDurableDefinitionCompletion")
            .finish_non_exhaustive()
    }
}
