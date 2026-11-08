//! Observe the public capture boundary through the actual Store-backed medium.
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{DurableSignalOwnerIdentity, SignalDurableJournal};
use worth_signal::facade::durability::*;

pub(super) struct PublicationProbe {
    pub(super) port: SignalDurableBranchMutationPort<(), (), (), (), ()>,
    pub(super) source: SignalDurableBranchSourcePort<(), (), ()>,
    pub(super) head: SignalDurableBranchDescriptor,
}
#[derive(Default)]
pub(super) struct CaptureObservation {
    pub(super) publication: Mutex<Option<PublicationProbe>>,
    pub(super) refused: AtomicUsize,
    pub(super) effects: AtomicUsize,
    pub(super) fail_read_once: AtomicBool,
}
struct Source {
    original: Box<dyn SignalDurableReadSource>,
    observation: Arc<CaptureObservation>,
}
impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ObservedStoreSignalSource")
    }
}
impl SignalDurableReadSource for Source {
    fn owner(&self) -> DurableSignalOwnerIdentity {
        self.original.owner()
    }
    fn identity_contribution(&self) -> SignalRecordedIdentityContribution {
        self.original.identity_contribution()
    }
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalDurableMediumDenial> {
        self.original.frontier(journal, execution)
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, SignalDurableMediumDenial> {
        if let Some(probe) = self.observation.publication.lock().unwrap().as_ref() {
            let head = probe.source.readmit_descriptor(&probe.head).unwrap();
            let result = probe.port.begin_advance(*execution, &head, &mut (), |_| {
                self.observation.effects.fetch_add(1, Ordering::Relaxed);
                Ok(())
            });
            assert!(
                matches!(result, Err(SignalDurableAdvanceDenial::CheckpointCaptureInProgress { owner }) if owner == self.owner())
            );
            self.observation.refused.fetch_add(1, Ordering::Relaxed);
        }
        if self
            .observation
            .fail_read_once
            .swap(false, Ordering::Relaxed)
        {
            return Err(SignalDurableMediumDenial::Unavailable);
        }
        self.original.read(journal, position, execution)
    }
}
pub(super) struct ObservedMedium {
    original: Arc<dyn SignalDurableMedium>,
    pub(super) observation: Arc<CaptureObservation>,
}
impl std::fmt::Debug for ObservedMedium {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ObservedStoreSignalMedium")
    }
}
impl ObservedMedium {
    pub(super) fn new(original: Arc<dyn SignalDurableMedium>) -> Arc<Self> {
        Arc::new(Self {
            original,
            observation: Arc::new(CaptureObservation::default()),
        })
    }
}
impl SignalDurableMedium for ObservedMedium {
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
        Ok(Box::new(Source {
            original: self.original.capture_recovery_source(execution)?,
            observation: Arc::clone(&self.observation),
        }))
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
        self.original.advance_branch(artifact, execution)
    }
    fn retire(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalDurableMediumDenial> {
        self.original.retire(execution)
    }
}
