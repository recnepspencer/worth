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
    foreign: Option<Arc<dyn SignalDurableMedium>>,
    pub(super) retry_checked: AtomicBool,
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
            foreign: None,
            retry_checked: AtomicBool::new(false),
        })
    }
    pub(super) fn with_foreign(
        original: Arc<dyn SignalDurableMedium>,
        foreign: Arc<dyn SignalDurableMedium>,
    ) -> Arc<Self> {
        Arc::new(Self {
            original,
            foreign: Some(foreign),
            pause_branch: AtomicBool::new(true),
            branch_observed: AtomicBool::new(false),
            retry_checked: AtomicBool::new(false),
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
    fn retry_definition_after_no_effect(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        proof: &worth_proof::ProvenNoEffectPublication<SignalNoEffectOriginal>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        if let Some(foreign) = &self.foreign {
            let wrong =
                super::super::media::definition(execution, artifact.identity().ordinal() + 1, None);
            subject_refusal(
                self.original
                    .retry_definition_after_no_effect(&wrong, proof, execution),
            );
            owner_refusal(foreign.retry_definition_after_no_effect(artifact, proof, execution));
            self.retry_checked.store(true, Ordering::Relaxed);
        }
        let result = self
            .original
            .retry_definition_after_no_effect(artifact, proof, execution);
        if matches!(&result, SignalPublicationObservation::Finished(done) if matches!(done.fate(), worth_foundational::facade::DurablePublicationFate::Committed))
        {
            same_commit(
                &result,
                self.original
                    .retry_definition_after_no_effect(artifact, proof, execution),
            );
        }
        result
    }
    fn retry_branch_after_no_effect(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        proof: &worth_proof::ProvenNoEffectPublication<SignalNoEffectOriginal>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation {
        if let Some(foreign) = &self.foreign {
            let definition = super::super::media::definition(execution, 1001, None);
            let wrong = super::super::media::advance(execution, &definition);
            subject_refusal(
                self.original
                    .retry_branch_after_no_effect(&wrong, proof, execution),
            );
            owner_refusal(foreign.retry_branch_after_no_effect(artifact, proof, execution));
            self.retry_checked.store(true, Ordering::Relaxed);
        }
        let result = self
            .original
            .retry_branch_after_no_effect(artifact, proof, execution);
        if matches!(&result, SignalPublicationObservation::Finished(done) if matches!(done.fate(), worth_foundational::facade::DurablePublicationFate::Committed))
        {
            same_commit(
                &result,
                self.original
                    .retry_branch_after_no_effect(artifact, proof, execution),
            );
        }
        result
    }
}
fn subject_refusal(result: SignalPublicationObservation) {
    assert!(
        matches!(result, SignalPublicationObservation::Finished(done) if matches!(done.fate(),
        worth_foundational::facade::DurablePublicationFate::ProvenNoEffect { detail: SignalDurableMediumDenial::PublicationRetrySubjectMismatch { .. } }))
    );
}
fn owner_refusal(result: SignalPublicationObservation) {
    assert!(
        matches!(result, SignalPublicationObservation::Finished(done) if matches!(done.fate(),
        worth_foundational::facade::DurablePublicationFate::ProvenNoEffect { detail: SignalDurableMediumDenial::PublicationRetryOwnerMismatch { observed: None, .. } }))
    );
}
fn same_commit(first: &SignalPublicationObservation, repeated: SignalPublicationObservation) {
    let SignalPublicationObservation::Finished(first) = first else {
        panic!("expected original completion")
    };
    let SignalPublicationObservation::Finished(repeated) = repeated else {
        panic!("committed original cannot become pending")
    };
    assert_eq!(first.committed_location(), repeated.committed_location());
    assert!(matches!(
        repeated.fate(),
        worth_foundational::facade::DurablePublicationFate::Committed
    ));
}
