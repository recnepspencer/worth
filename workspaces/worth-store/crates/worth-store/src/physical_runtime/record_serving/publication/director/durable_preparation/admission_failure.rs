use super::*;

pub(super) fn map_idempotency_admission(
    error: PhysicalMutationIdempotencyRegistryAdmissionError<
        PhysicalMutationIdentityReservationError,
    >,
) -> PhysicalMutationPreparationOutcome {
    match error {
        PhysicalMutationIdempotencyRegistryAdmissionError::Reservation(error) => {
            map_identity_reservation(error)
        }
        PhysicalMutationIdempotencyRegistryAdmissionError::Denied(denial) => {
            map_idempotency_denial(denial)
        }
    }
}

fn map_idempotency_denial(
    denial: PhysicalMutationIdempotencyRegistryDenial,
) -> PhysicalMutationPreparationOutcome {
    match denial {
        PhysicalMutationIdempotencyRegistryDenial::AuthorityReleased => {
            TransitionOutcome::stale(PhysicalMutationPreparationStale::DurabilityAuthorityReleased)
                .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::ForeignStore
        | PhysicalMutationIdempotencyRegistryDenial::ForeignMutationStore => {
            TransitionOutcome::rebind_required(
                PhysicalMutationPreparationRebindRequired::ForeignStore,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::ForeignPolicy => {
            TransitionOutcome::rebind_required(
                PhysicalMutationPreparationRebindRequired::ForeignDurabilityPolicy,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::ForeignMutationRuntime => {
            TransitionOutcome::rebind_required(
                PhysicalMutationPreparationRebindRequired::ForeignRuntime,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::Expired => {
            TransitionOutcome::denied(PhysicalMutationPreparationDenial::IdempotencyExpired).into()
        }
        PhysicalMutationIdempotencyRegistryDenial::Conflict => {
            TransitionOutcome::denied(PhysicalMutationPreparationDenial::IdempotencyConflict).into()
        }
        PhysicalMutationIdempotencyRegistryDenial::PendingUnresolvedLimitReached => {
            TransitionOutcome::deferred(
                PhysicalMutationPreparationDeferred::PendingUnresolvedLimitReached,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::LiveBindingLimitReached => {
            TransitionOutcome::deferred(
                PhysicalMutationPreparationDeferred::LiveBindingLimitReached,
            )
            .into()
        }
    }
}

fn map_identity_reservation(
    error: PhysicalMutationIdentityReservationError,
) -> PhysicalMutationPreparationOutcome {
    match error {
        PhysicalMutationIdentityReservationError::Stale(stale) => {
            let stale = match stale {
                PhysicalWorkSubmissionStale::OwnerReleased => {
                    PhysicalMutationPreparationStale::WorkOwnerReleased
                }
                PhysicalWorkSubmissionStale::LifecycleGenerationAdvanced => {
                    PhysicalMutationPreparationStale::LifecycleGenerationAdvanced
                }
                PhysicalWorkSubmissionStale::AdmissionStopped => {
                    PhysicalMutationPreparationStale::AdmissionStopped
                }
                PhysicalWorkSubmissionStale::SignalOwnerUnavailable => {
                    PhysicalMutationPreparationStale::SignalOwnerUnavailable
                }
            };
            TransitionOutcome::stale(stale).into()
        }
        PhysicalMutationIdentityReservationError::Failed(
            PhysicalWorkSubmissionFailure::OperationIdentityExhausted,
        ) => TransitionOutcome::failed(
            PhysicalMutationPreparationFailure::OperationIdentityExhausted,
        )
        .into(),
    }
}
