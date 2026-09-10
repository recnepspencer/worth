use crate::publication::RuntimeWorldCancellationToken;
use crate::publication::{NoEffectCause, SignalAttemptProgress, SignalComponentPlanPosture};
use crate::recovery::ProductUnpublishedCause;

use super::RuntimeWorldOwnerRoot;

use worth_signal::facade::branch::{SignalBranchAdvanceDenial, SignalBranchForkOperationDenial};
use worth_signal::facade::{SignalError, SignalTransaction};

pub(super) struct SignalExecutionFailure {
    pub(super) cause: ProductUnpublishedCause,
    pub(super) no_effect: NoEffectCause,
    pub(super) partial: SignalAttemptProgress,
}

/// The Signal borrow one publication is allowed to hold. The advancing arm
/// carries the caller's own mutation body unboxed, so the execution seam never
/// erases the closure it was handed.
pub(super) enum SignalExecutionRequest<'a, Ctx, F, H> {
    RetainExact,
    AdvanceExact {
        runtime_ctx: &'a mut Ctx,
        apply: F,
        admit_activation: H,
    },
    PublishConditionalDefinition {
        publication: SignalDefinitionPublicationAdmission,
        runtime_ctx: &'a mut Ctx,
        apply: F,
        admit_activation: H,
    },
}

pub(super) enum SignalDefinitionPublicationAdmission {
    Prepared(worth_signal::facade::branch::SignalConditionalDefinitionPublicationOperation),
    Admitted(worth_signal::facade::branch::AdmittedSignalConditionalDefinitionPublication),
}

/// The mutation type used when a publication declares it will not touch the
/// Signal owner. It names a body that cannot exist, so `RetainExact` is the
/// only inhabited request on that path.
pub(super) type UntouchedSignalMutation<D, I, E, Ctx, T> =
    fn(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The Signal publication leg. Publication either retains the exact
    /// component basis or advances it exactly once; creating a branch is a
    /// separate owner operation and never reaches this path.
    pub(super) fn execute_signal<F, H>(
        &self,
        attempt: &mut crate::publication::ReservedCompositePublicationAttempt,
        progress: &mut crate::publication::CompositeAttemptProgress,
        request: SignalExecutionRequest<'_, Ctx, F, H>,
        runtime_cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<(), SignalExecutionFailure>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
        H: FnOnce(
            Option<worth_signal::facade::branch::SignalConditionalDefinitionAdvanceBinding>,
        ) -> Result<(), SignalError>,
    {
        match (attempt.plan().signal().posture(), request) {
            (SignalComponentPlanPosture::RetainExact, _) => Ok(()),
            (
                SignalComponentPlanPosture::AdvanceExact,
                SignalExecutionRequest::AdvanceExact {
                    runtime_ctx,
                    apply,
                    admit_activation,
                },
            ) => {
                // SPEC-P4-005: the Signal owner is handed the token embedded in
                // the Runtime World source, never a caller-supplied Signal
                // token, so one `cancel()` reaches an in-flight advance.
                attempt.counters_mut().record_signal_owner_contact();
                let signal_cancellation = runtime_cancellation.signal_token();
                #[cfg(test)]
                super::rehearsal::reach_signal_advance(self.owner_identity(), signal_cancellation);
                let mut completion = self
                    .state
                    .signal
                    .mutation_port()
                    .advance_exact_with_completion(
                        attempt.plan().signal().expected(),
                        runtime_ctx,
                        signal_cancellation,
                        apply,
                    );
                let result = completion.take_result();
                match result {
                    Some(Ok(mut outcome)) => {
                        let binding = outcome.take_conditional_definition_advance_binding();
                        progress.set_signal(SignalAttemptProgress::advanced(outcome));
                        attempt.record_progress(progress);
                        if admit_activation(binding).is_err() {
                            completion.resume_unwind();
                            return Err(activation_failure(progress.signal().retained_image()));
                        }
                        completion.resume_unwind();
                        Ok(())
                    }
                    Some(Err(denial)) => {
                        completion.resume_unwind();
                        Err(advance_failure(&denial))
                    }
                    None => {
                        completion.resume_unwind();
                        unreachable!("missing owner result carries an unwind")
                    }
                }
            }
            (
                SignalComponentPlanPosture::AdvanceExact,
                SignalExecutionRequest::PublishConditionalDefinition {
                    publication,
                    runtime_ctx,
                    apply,
                    admit_activation,
                },
            ) => {
                attempt.counters_mut().record_signal_owner_contact();
                let signal_cancellation = runtime_cancellation.signal_token();
                let SignalDefinitionPublicationAdmission::Admitted(publication) = publication
                else {
                    unreachable!("definition publication is admitted before owner execution")
                };
                let mut completion = self
                    .state
                    .signal_definition_publication
                    .advance_exact_with_completion(
                        publication,
                        attempt.plan().signal().expected(),
                        runtime_ctx,
                        signal_cancellation,
                        apply,
                    );
                let result = completion.take_result();
                match result {
                    Some(Ok(mut outcome)) => {
                        let binding = outcome.take_conditional_definition_advance_binding();
                        progress.set_signal(SignalAttemptProgress::advanced(outcome));
                        attempt.record_progress(progress);
                        if admit_activation(binding).is_err() {
                            completion.resume_unwind();
                            return Err(activation_failure(progress.signal().retained_image()));
                        }
                        completion.resume_unwind();
                        Ok(())
                    }
                    Some(Err(denial)) => {
                        completion.resume_unwind();
                        Err(advance_failure(&denial))
                    }
                    None => {
                        completion.resume_unwind();
                        unreachable!("missing owner result carries an unwind")
                    }
                }
            }
            // An advancing plan reached the seam without the caller's Signal
            // borrow. The plan and the borrow are chosen together at the
            // typestate, so this is an owner the publication cannot reach.
            (SignalComponentPlanPosture::AdvanceExact, SignalExecutionRequest::RetainExact) => {
                Err(SignalExecutionFailure {
                    cause: ProductUnpublishedCause::SiblingOwnerDenied,
                    no_effect: NoEffectCause::OwnerUnavailable,
                    partial: SignalAttemptProgress::untouched(),
                })
            }
        }
    }
}

fn activation_failure(partial: SignalAttemptProgress) -> SignalExecutionFailure {
    SignalExecutionFailure {
        cause: ProductUnpublishedCause::SiblingOwnerDenied,
        no_effect: NoEffectCause::PreEffectFailure,
        partial,
    }
}

fn advance_failure(denial: &SignalBranchAdvanceDenial) -> SignalExecutionFailure {
    let cancellation = matches!(denial, SignalBranchAdvanceDenial::CancelledNoMovement);
    SignalExecutionFailure {
        cause: if cancellation {
            ProductUnpublishedCause::CancellationAfterEffect
        } else {
            ProductUnpublishedCause::SiblingOwnerDenied
        },
        no_effect: map_advance_no_effect(denial),
        partial: SignalAttemptProgress::untouched(),
    }
}

pub(crate) fn map_fork_no_effect(denial: &SignalBranchForkOperationDenial) -> NoEffectCause {
    match denial {
        SignalBranchForkOperationDenial::CancelledNoMovement => {
            NoEffectCause::CancelledBeforeEffect
        }
        SignalBranchForkOperationDenial::OperationCapacityExhausted { .. }
        | SignalBranchForkOperationDenial::LiveBranchCapacityExhausted { .. }
        | SignalBranchForkOperationDenial::ReservationCapacityExhausted { .. }
        | SignalBranchForkOperationDenial::RetentionUnavailable { .. } => {
            NoEffectCause::CapacityExhausted
        }
        SignalBranchForkOperationDenial::OwnerUnavailable(_) => NoEffectCause::OwnerUnavailable,
        _ => NoEffectCause::PreEffectFailure,
    }
}

pub(super) fn map_advance_no_effect(denial: &SignalBranchAdvanceDenial) -> NoEffectCause {
    match denial {
        SignalBranchAdvanceDenial::CancelledNoMovement => NoEffectCause::CancelledBeforeEffect,
        SignalBranchAdvanceDenial::OperationCapacityExhausted { .. }
        | SignalBranchAdvanceDenial::RetentionUnavailable { .. } => {
            NoEffectCause::CapacityExhausted
        }
        SignalBranchAdvanceDenial::OwnerUnavailable(_) => NoEffectCause::OwnerUnavailable,
        _ => NoEffectCause::PreEffectFailure,
    }
}
