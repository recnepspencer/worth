//! Store's Signal adapter: two registered journals, opaque owner bytes.
mod attachment;
mod home_slot;
mod initial_source;
pub use attachment::StoreSignalMediumAttachmentRefused;
pub(crate) use home_slot::UnclaimedSignalSlot;
pub use initial_source::HomeSignalInitialSourceDenial;
pub(crate) use initial_source::{HomeInitialSignalSource, OriginalSignalInitialSource};
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
    home_source: Mutex<Option<crate::checkpoint::SelectedHomeSource>>,
    placement: AdmittedRecordPlacementPolicy,
    publications: SignalPublications,
    admission: SignalDurableOwnerAdmissionRegistry,
    checkpoint: Option<worth_signal::facade::durability::SignalAuthoredHomeImage>,
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
        checkpoint: Option<worth_signal::facade::durability::SignalAuthoredHomeImage>,
        placement: AdmittedRecordPlacementPolicy,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(Arc<ServingPhysicalRuntime>, Arc<Self>), StoreSignalMediumAttachmentRefused> {
        let prepared = Self::prepare_attachment(&serving, &binding, checkpoint.as_ref(), execution);
        let (funding, admission) = match prepared {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err(StoreSignalMediumAttachmentRefused::new(
                    cause, serving, binding, checkpoint, placement,
                ))
            }
        };
        let serving: Arc<ServingPhysicalRuntime> = Arc::from(serving);
        let medium = Arc::new(Self {
            serving: Mutex::new(Some(Arc::clone(&serving))),
            binding,
            checkpoint,
            home_source: Mutex::new(None),
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
        checkpoint: Option<&worth_signal::facade::durability::SignalAuthoredHomeImage>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Option<ExecutionMemoryReservation>,
            SignalDurableOwnerAdmissionRegistry,
        ),
        D,
    > {
        if checkpoint.is_some_and(|image| image.owner() != binding.owner()) {
            return Err(D::ForeignOwner);
        }
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
    pub(crate) fn physical_close_holder(
        &self,
        actual: &Arc<ServingPhysicalRuntime>,
    ) -> crate::close::HomeSignalPhysicalHolder {
        use crate::close::HomeSignalPhysicalHolder as Holder;
        match self.serving.try_lock() {
            Ok(share) => match share.as_ref() {
                Some(share) if Arc::ptr_eq(share, actual) => {
                    Holder::Retaining(self.binding.owner())
                }
                Some(_) | None => Holder::Released(self.binding.owner()),
            },
            Err(_) => Holder::InspectionBusy(self.binding.owner()),
        }
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
        retry: Option<
            &worth_proof::ProvenNoEffectPublication<
                worth_signal::facade::durability::SignalNoEffectOriginal,
            >,
        >,
    ) -> SignalPublicationObservation {
        if artifact.owner() != self.binding.owner() {
            return publication::no_effect(D::ForeignOwner);
        }
        let subject = subject::SignalPublicationSubject::new(artifact);
        publication::publish(self, &subject, execution, retry)
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
            None,
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
            None,
        )
    }
    fn retry_definition_after_no_effect(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        proof: &worth_proof::ProvenNoEffectPublication<
            worth_signal::facade::durability::SignalNoEffectOriginal,
        >,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.publish(
            subject::SignalPublicationArtifact::Definition(artifact.clone()),
            execution,
            Some(proof),
        )
    }
    fn retry_branch_after_no_effect(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        proof: &worth_proof::ProvenNoEffectPublication<
            worth_signal::facade::durability::SignalNoEffectOriginal,
        >,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        self.publish(
            subject::SignalPublicationArtifact::Advance(artifact.clone()),
            execution,
            Some(proof),
        )
    }
    fn retire(&self, execution: &ExecutionRequest<'_, '_>) -> Result<(), D> {
        match self.publications.posture() {
            SignalPublicationTaskPosture::Busy => return Err(D::PublicationBusy),
            SignalPublicationTaskPosture::Unsettled => return Err(D::PublicationUnresolved),
            SignalPublicationTaskPosture::Terminal => {}
        }
        let mut serving = self.serving.try_lock().map_err(|_| D::PublicationBusy)?;
        if serving.is_none() {
            return Ok(());
        }
        execution
            .check_live()
            .map_err(|cause| D::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        self.admission.retire()?;
        let original = serving.take();
        drop(serving);
        drop(original);
        Ok(())
    }
}
