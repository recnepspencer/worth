//! Durable opening precedes the caller's one Native candidate preparation.
use super::*;
impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Select the actual admitted Home binding. Only a live, consistently
    /// memory-owned World returns None; unavailable or foreign owners refuse.
    pub fn query_publication_home_binding(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<
        Option<worth_foundational::facade::DurableWorldOwnerIdentity>,
        crate::publication::WorldOpenAttemptRefusal,
    > {
        use crate::publication::WorldOpenAttemptRefusal as Refusal;
        let owner = self.service().map_err(|_| {
            Refusal::Reservation(NoEffectCompositePublication::new(
                NoEffectCause::OwnerUnavailable,
                Some(expected.clone()),
            ))
        })?;
        let identity = owner.owner_identity();
        match identity.admit_incarnation(expected.owner_identity()) {
            Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => {
                return Err(Refusal::Reservation(NoEffectCompositePublication::new(
                    NoEffectCause::StaleIncarnation,
                    Some(expected.clone()),
                )));
            }
            Err(worth_proof::HomeIncarnationDenial::ForeignHome) => {
                return Err(Refusal::IdentityScope)
            }
            Ok(()) => {}
        }
        // Home scope equality alone does not establish this process admission.
        if expected.owner_identity() != identity
            || expected.owner_identity().get() != identity.get()
        {
            return Err(Refusal::IdentityScope);
        }
        match (identity.durable(), owner.state.durable.as_ref()) {
            (None, None) => Ok(None),
            (Some(home), Some(binding)) if binding.binding().owner() == home => Ok(Some(home)),
            (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => Err(Refusal::IdentityScope),
        }
    }

    /// Return the original preparing bundle before any journal submission.
    /// The closed Query caller retains it outside its disposable request first.
    pub fn prepare_query_publication_without_signal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<
        crate::publication::WorldPendingOpenAttempt,
        crate::publication::WorldOpenAttemptRefusal,
    > {
        let owner = self.service().map_err(|_| {
            crate::publication::WorldOpenAttemptRefusal::Reservation(
                NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                ),
            )
        })?;
        owner.prepare_durable_open(execution, expected, change, cancellation, deadline)
    }

    pub fn open_publication_without_signal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> crate::publication::WorldOpenAttemptOutcome {
        match self.service() {
            Ok(owner) => {
                owner.begin_open_without_signal(execution, expected, change, cancellation, deadline)
            }
            Err(_) => crate::publication::WorldOpenAttemptOutcome::Refused(
                crate::publication::WorldOpenAttemptRefusal::Reservation(
                    NoEffectCompositePublication::new(
                        NoEffectCause::OwnerUnavailable,
                        Some(expected),
                    ),
                ),
            ),
        }
    }
}
