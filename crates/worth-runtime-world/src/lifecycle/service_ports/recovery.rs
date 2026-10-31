use super::super::ports::RuntimeWorldRecoveryService;
use super::super::RuntimeWorldOwnerUnavailable;
use crate::lifecycle::RuntimeWorldServiceDenial;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's recovery service.
#[derive(Clone)]
pub struct RuntimeWorldRecoveryPort {
    owner: Weak<dyn RuntimeWorldRecoveryService + Send + Sync>,
}
impl RuntimeWorldRecoveryPort {
    pub fn retained_commits(
        &self,
        candidates: &[crate::identity::CompositeCommitIdentity],
    ) -> Result<Vec<crate::identity::CompositeCommitIdentity>, RuntimeWorldOwnerUnavailable> {
        Ok(self.service()?.retained_commits(candidates))
    }
    pub fn retire_changed_history(
        &self,
    ) -> Result<
        Vec<crate::identity::CompositeCommitIdentity>,
        crate::history::HistoryReclamationDenial,
    > {
        self.service()
            .map_err(crate::history::HistoryReclamationDenial::OwnerUnavailable)?
            .retire_changed_history()
    }
    pub fn release_retired_branch_history(
        &self,
        from: &crate::identity::CompositeCommitIdentity,
    ) -> Result<crate::history::RetiredBranchHistoryRelease, crate::history::HistoryReclamationDenial>
    {
        self.service()
            .map_err(crate::history::HistoryReclamationDenial::OwnerUnavailable)?
            .release_retired_branch_history(from)
    }

    pub fn inspect_recovered_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
    ) -> Result<
        crate::recovery::WorldRecoveredUnpublishedEffects,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        self.service()?.inspect_recovered_effects(handle)
    }

    pub fn release_recovered_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: crate::recovery::WorldRecoveredUnpublishedEffects,
    ) -> crate::recovery::WorldRecoveredReleaseOutcome {
        match self.service() {
            Ok(owner) => owner.release_recovered_effects(execution, effects),
            Err(cause) => crate::recovery::WorldRecoveredReleaseOutcome::AdmissionRefused {
                cause: cause.into(),
                custody: effects,
            },
        }
    }
    pub fn prepare_settled_relational_adoption(
        &self,
        effects: &crate::recovery::ProductUnpublishedOwnerEffects,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
        deadline: Option<crate::lifecycle::RuntimeWorldInstant>,
    ) -> Result<
        crate::publication::PreparedCompositePublicationWithoutSignal,
        crate::recovery::RuntimeWorldSettledRelationalAdoptionDenial,
    > {
        self.service()
            .map_err(|unavailable| {
                crate::recovery::RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
                    unavailable.into(),
                )
            })?
            .prepare_settled_relational_adoption(effects, cancellation, deadline)
    }

    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldRecoveryService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldRecoveryService + Send + Sync>, RuntimeWorldOwnerUnavailable>
    {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn inspect_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
    ) -> Result<
        crate::recovery::ProductUnpublishedOwnerEffects,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        self.service()?.inspect_effects(handle)
    }
    pub fn release_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::recovery::ProductUnpublishedCleanup,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        self.service()?.release_effects(handle, minimum_age_ticks)
    }
    pub fn continue_effects(
        &self,
        effects: crate::recovery::ProductUnpublishedOwnerEffects,
    ) -> Result<
        crate::recovery::RecoveryContinuationContract,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        self.service()?.continue_effects(effects)
    }
    pub fn recover_performed(
        &self,
        identity: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        crate::publication::PerformedCompositePublication,
        RuntimeWorldServiceDenial<crate::recovery::PerformedPublicationRecoveryDenial>,
    > {
        self.service()?
            .recover_performed(identity)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
}
