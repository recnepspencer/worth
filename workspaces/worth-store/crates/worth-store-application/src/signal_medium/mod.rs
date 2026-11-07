//! Store's Signal adapter: two registered journals, opaque owner bytes.
mod attachment;
pub use attachment::StoreSignalMediumAttachmentRefused;
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
    SignalDurableMedium, SignalDurableMediumDenial as D, SignalDurableOwnerAdmissionRegistry,
    SignalDurableReadSource, SignalPublicationObservation, SignalPublicationTaskPosture,
};
use worth_store::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};

pub struct StoreSignalMedium {
    serving: Mutex<Option<Arc<ServingPhysicalRuntime>>>,
    binding: SignalDurableHomeBinding,
    placement: AdmittedRecordPlacementPolicy,
    publications: SignalPublications,
    admission: SignalDurableOwnerAdmissionRegistry,
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
    /// Consume the original unshared physical allocation with Signal's journals
    /// registered. The returned serving share cannot become another owned input
    /// until this adapter releases its share. A refusal retains that original.
    pub fn from_serving(
        serving: Box<ServingPhysicalRuntime>,
        binding: SignalDurableHomeBinding,
        placement: AdmittedRecordPlacementPolicy,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(Arc<ServingPhysicalRuntime>, Arc<Self>), StoreSignalMediumAttachmentRefused> {
        let prepared = Self::prepare_attachment(&serving, &binding, execution);
        let (funding, admission) = match prepared {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err(StoreSignalMediumAttachmentRefused::new(
                    cause, serving, binding, placement,
                ))
            }
        };
        let serving: Arc<ServingPhysicalRuntime> = Arc::from(serving);
        let medium = Arc::new(Self {
            serving: Mutex::new(Some(Arc::clone(&serving))),
            binding,
            placement,
            publications: SignalPublications::default(),
            admission,
            _funding: funding,
        });
        Ok((serving, medium))
    }
    fn prepare_attachment(
        serving: &ServingPhysicalRuntime,
        binding: &SignalDurableHomeBinding,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Option<ExecutionMemoryReservation>,
            SignalDurableOwnerAdmissionRegistry,
        ),
        D,
    > {
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
        let admission = SignalDurableOwnerAdmissionRegistry::new(binding, execution)?;
        Ok((funding, admission))
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
    fn owner_admission_registry(&self) -> &SignalDurableOwnerAdmissionRegistry {
        &self.admission
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
        self.admission.retire()?;
        let original = serving.take();
        drop(serving);
        drop(original);
        Ok(())
    }
}
