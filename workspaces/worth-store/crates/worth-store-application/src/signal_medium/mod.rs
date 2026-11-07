//! Store's Signal adapter: two registered journals, opaque owner bytes.
mod publication;
mod publication_fate;
mod publication_task;
#[cfg(test)]
mod reservation_tests;
mod source;
mod subject;
use publication_task::SignalPublications;
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_signal::facade::durability::{
    SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact, SignalDurableHomeBinding,
    SignalDurableMedium, SignalDurableMediumDenial as D, SignalDurableReadSource,
    SignalPublicationObservation, SignalPublicationTaskPosture,
};
use worth_store::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};

pub struct StoreSignalMedium {
    serving: Mutex<Option<Arc<ServingPhysicalRuntime>>>,
    binding: SignalDurableHomeBinding,
    placement: AdmittedRecordPlacementPolicy,
    publications: SignalPublications,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for StoreSignalMedium {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreSignalMedium")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}
impl StoreSignalMedium {
    /// Composition requires the actual serving allocation with Signal's two
    /// journals already registered. This creates transport, never a Signal
    /// owner or Home readmission witness.
    pub fn from_serving(
        serving: Arc<ServingPhysicalRuntime>,
        binding: SignalDurableHomeBinding,
        placement: AdmittedRecordPlacementPolicy,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, D> {
        for journal in [
            worth_foundational::facade::SignalDurableJournal::Definitions,
            worth_foundational::facade::SignalDurableJournal::Advances,
        ] {
            serving
                .ordered_journal(source::name(journal))
                .map_err(|cause| D::Storage(format!("{cause:?}")))?;
        }
        let funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64,
            )
            .map_err(D::Request)?;
        Ok(Arc::new(Self {
            serving: Mutex::new(Some(serving)),
            binding,
            placement,
            publications: SignalPublications::default(),
            _funding: funding,
        }))
    }
    fn serving(&self) -> Result<Arc<ServingPhysicalRuntime>, D> {
        self.serving
            .try_lock()
            .map_err(|_| D::PublicationBusy)?
            .as_ref()
            .cloned()
            .ok_or(D::Unavailable)
    }
    fn publish(
        &self,
        artifact: subject::SignalPublicationArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        if artifact.owner() != self.binding.owner() {
            return publication::no_effect(D::ForeignOwner);
        }
        let subject = subject::SignalPublicationSubject::new(artifact);
        publication::publish(self, &subject, execution)
    }
}
impl SignalDurableMedium for StoreSignalMedium {
    fn binding(&self) -> &SignalDurableHomeBinding {
        &self.binding
    }
    fn publication_task_posture(&self) -> SignalPublicationTaskPosture {
        self.publications.posture()
    }
    fn capture_recovery_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, D> {
        source::capture(self, execution)
    }
    fn advance_definition(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.publish(
            subject::SignalPublicationArtifact::Definition(artifact.clone()),
            execution,
        )
    }
    fn advance_branch(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.publish(
            subject::SignalPublicationArtifact::Advance(artifact.clone()),
            execution,
        )
    }
    fn retire(&self, execution: &ExecutionRequest<'_, '_>) -> Result<(), D> {
        match self.publications.posture() {
            SignalPublicationTaskPosture::Busy => return Err(D::PublicationBusy),
            SignalPublicationTaskPosture::Unsettled => return Err(D::PublicationUnresolved),
            SignalPublicationTaskPosture::Terminal => {}
        }
        execution.consult().map_err(D::Request)?;
        let mut serving = self.serving.try_lock().map_err(|_| D::PublicationBusy)?;
        let original = serving.take();
        drop(serving);
        drop(original);
        Ok(())
    }
}
