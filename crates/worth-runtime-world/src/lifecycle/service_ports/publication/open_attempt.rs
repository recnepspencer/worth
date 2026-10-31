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
