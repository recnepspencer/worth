//! Borrowed observations pause before the real original branch publication.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::*;

pub(super) struct PausedMedium {
    original: Arc<dyn SignalDurableMedium>,
    pub(super) pause_branch: AtomicBool,
    pub(super) branch_observed: AtomicBool,
}
impl std::fmt::Debug for PausedMedium {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PausedOriginalStoreMedium")
    }
}
impl PausedMedium {
    pub(super) fn new(original: Arc<dyn SignalDurableMedium>) -> Arc<Self> {
        Arc::new(Self {
            original,
            pause_branch: AtomicBool::new(true),
            branch_observed: AtomicBool::new(false),
        })
    }
}
impl SignalDurableMedium for PausedMedium {
    fn binding(&self) -> &SignalDurableHomeBinding {
        self.original.binding()
    }
    fn owner_admission_registry(&self) -> &SignalDurableOwnerAdmissionRegistry {
        self.original.owner_admission_registry()
    }
    fn publication_task_posture(&self) -> SignalPublicationTaskPosture {
        self.original.publication_task_posture()
    }
    fn capture_recovery_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, SignalDurableMediumDenial> {
        self.original.capture_recovery_source(execution)
    }
    fn advance_definition(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.original.advance_definition(artifact, execution)
    }
    fn advance_branch(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.branch_observed.store(true, Ordering::Relaxed);
        if self.pause_branch.load(Ordering::Relaxed) {
            return SignalPublicationObservation::Pending;
        }
        self.original.advance_branch(artifact, execution)
    }
    fn retire(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalDurableMediumDenial> {
        self.original.retire(execution)
    }
}
