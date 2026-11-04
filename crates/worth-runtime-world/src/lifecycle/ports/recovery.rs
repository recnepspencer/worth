use super::*;

/// Shared internal seam for retained owner effects. Recovery never fabricates
/// a product commit or promotes a partial record to performed publication.
pub(crate) trait RuntimeWorldRecoveryService:
    crate::lifecycle::availability::RuntimeWorldAvailability
{
    fn retained_commits(
        &self,
        candidates: &[crate::identity::CompositeCommitIdentity],
    ) -> Vec<crate::identity::CompositeCommitIdentity>;
    fn retire_changed_history(
        &self,
    ) -> Result<Vec<crate::identity::CompositeCommitIdentity>, HistoryReclamationDenial>;
    fn release_retired_branch_history(
        &self,
        from: &crate::identity::CompositeCommitIdentity,
    ) -> Result<crate::history::RetiredBranchHistoryRelease, HistoryReclamationDenial>;
    fn inspect_recovered_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
    ) -> Result<
        crate::recovery::WorldRecoveredUnpublishedEffects,
        crate::recovery::RuntimeWorldRecoveryDenial,
    >;
    fn continue_recovered_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: crate::recovery::WorldRecoveredUnpublishedEffects,
    ) -> crate::recovery::WorldRecoveredContinuationOutcome;
    fn release_recovered_effects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: crate::recovery::WorldRecoveredUnpublishedEffects,
    ) -> crate::recovery::WorldRecoveredReleaseOutcome;
    fn prepare_settled_relational_adoption(
        &self,
        effects: &ProductUnpublishedOwnerEffects,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<
        crate::publication::PreparedCompositePublicationWithoutSignal,
        crate::recovery::RuntimeWorldSettledRelationalAdoptionDenial,
    >;

    fn inspect_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
    ) -> Result<ProductUnpublishedOwnerEffects, crate::recovery::RuntimeWorldRecoveryDenial>;
    fn release_effects(
        &self,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::recovery::ProductUnpublishedCleanup,
        crate::recovery::RuntimeWorldRecoveryDenial,
    >;

    fn continue_effects(
        &self,
        effects: ProductUnpublishedOwnerEffects,
    ) -> Result<RecoveryContinuationContract, crate::recovery::RuntimeWorldRecoveryDenial>;
    fn recover_performed(
        &self,
        identity: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        crate::publication::PerformedCompositePublication,
        crate::recovery::PerformedPublicationRecoveryDenial,
    >;
}
