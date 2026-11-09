//! Execute the original durable Signal leg under its installed World affinity.
use super::{RuntimeWorldOwnerRoot, SignalExecutionFailure};
use crate::publication::durable_signal::{WorldDurableSignalBacking, WorldDurableSignalProgress};
use crate::publication::{
    CompositeAttemptProgress, NoEffectCause, ReservedCompositePublicationAttempt,
    SignalAttemptProgress,
};
use crate::recovery::ProductUnpublishedCause;
use worth_signal::facade::{SignalError, SignalTransaction};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn execute_durable_signal<F, H>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        attempt: &mut ReservedCompositePublicationAttempt,
        progress: &mut CompositeAttemptProgress,
        attachment: &crate::lifecycle::RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>,
        runtime_ctx: &mut Ctx,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
        apply: F,
        admit_activation: H,
    ) -> Result<(), SignalExecutionFailure>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
        H: FnOnce(
            Option<worth_signal::facade::branch::SignalConditionalDefinitionAdvanceBinding>,
        ) -> Result<(), SignalError>,
    {
        let failure = || SignalExecutionFailure {
            cause: ProductUnpublishedCause::SiblingOwnerDenied,
            no_effect: NoEffectCause::PreEffectFailure,
            partial: SignalAttemptProgress::untouched(),
        };
        let association = attempt.durable_association().ok_or_else(failure)?;
        let mutation = attachment.services.mutation_port();
        let binding = mutation
            .prepare_composite_binding(&execution, attempt.plan().signal().expected(), association)
            .map_err(|_| failure())?;
        let backing = WorldDurableSignalBacking::reserve(&execution).map_err(|_| failure())?;
        let grant = attachment.issuer.admit_original(binding.commitment());
        attempt.counters_mut().record_signal_owner_contact();
        let task = mutation
            .begin_composite_advance(
                execution,
                binding,
                grant,
                runtime_ctx,
                cancellation.signal_token(),
                apply,
            )
            .map_err(|denial| match denial {
                worth_signal::facade::durability::SignalDurableAdvanceDenial::Mutation(cause) => {
                    super::signal::advance_failure(&cause)
                }
                _ => failure(),
            })?;
        let retained = backing.retain(task);
        // Before the first durable drive, publish the exact task into original custody.
        progress.set_signal(SignalAttemptProgress::unresolved(std::sync::Arc::clone(
            &retained,
        )));
        attempt.record_progress(progress);
        match retained.drive(&execution) {
            WorldDurableSignalProgress::Committed(output) => {
                progress.set_signal(SignalAttemptProgress::durable(output));
                attempt.record_progress(progress);
                if admit_activation(None).is_err() {
                    return Err(SignalExecutionFailure {
                        partial: progress.signal().retained_image(),
                        ..failure()
                    });
                }
                Ok(())
            }
            WorldDurableSignalProgress::ProvenNoEffect => {
                progress.set_signal(SignalAttemptProgress::untouched());
                attempt.record_progress(progress);
                Err(failure())
            }
            WorldDurableSignalProgress::Pending | WorldDurableSignalProgress::Indeterminate => {
                Err(SignalExecutionFailure {
                    cause: ProductUnpublishedCause::SettlementPending,
                    no_effect: NoEffectCause::PreEffectFailure,
                    partial: progress.signal().retained_image(),
                })
            }
        }
    }
}
