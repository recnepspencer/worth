use super::*;
use crate::publication::{
    WorldDurablePublicationDenial as Denial, WorldDurablePublicationOutcome as Outcome,
    WorldPreparedPublicationWithoutSignal,
};

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Execute the same durable open with its original Signal intent.
    pub fn execute_durable_with_signal<F>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: crate::publication::WorldPreparedPublicationWithSignal,
        runtime_ctx: &mut Ctx,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
    ) -> Outcome
    where
        F: FnOnce(
            &mut worth_signal::facade::SignalTransaction<'_, D, I, E, Ctx, T>,
        ) -> Result<(), worth_signal::facade::SignalError>,
    {
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => {
                return Outcome::RecoveryRequired {
                    cause: Denial::OwnerUnavailable,
                    custody: prepared.cancel(),
                }
            }
        };
        if let Err(cause) = owner
            .owner_identity()
            .admit_incarnation(prepared.expected_head().owner_identity())
        {
            return Outcome::RecoveryRequired {
                cause: match cause {
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        Denial::StaleIncarnation
                    }
                    worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignOwner,
                },
                custody: prepared.cancel(),
            };
        }
        let outcome = owner.execute_with_signal_with_cutover(
            execution,
            prepared.prepared,
            runtime_ctx,
            cancellation,
            apply,
            None,
        );
        owner.finish_durable_publication(&execution, outcome, cancellation)
    }

    /// Execute the one associated preparation, select its durable occurrence,
    /// then complete that same cell movement. Every refusal retains the open.
    pub fn execute_durable_without_signal(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: WorldPreparedPublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Outcome {
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => {
                return Outcome::RecoveryRequired {
                    cause: Denial::OwnerUnavailable,
                    custody: prepared.cancel(),
                }
            }
        };
        if let Err(cause) = owner
            .owner_identity()
            .admit_incarnation(prepared.expected_head().owner_identity())
        {
            return Outcome::RecoveryRequired {
                cause: match cause {
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        Denial::StaleIncarnation
                    }
                    worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignOwner,
                },
                custody: prepared.cancel(),
            };
        }
        let outcome = owner.execute_without_signal_with_cutover(
            execution,
            prepared.prepared,
            cancellation,
            None,
        );
        owner.finish_durable_publication(&execution, outcome, cancellation)
    }
}
