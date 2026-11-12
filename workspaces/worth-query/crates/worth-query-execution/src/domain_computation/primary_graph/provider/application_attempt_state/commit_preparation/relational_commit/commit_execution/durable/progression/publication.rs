//! Publish or retain the original World outcome after all preparation frames return.
use super::*;

#[inline(never)]
pub(super) fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    guard: &mut QueryDurablePreparationGuard,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Progress, worth_execution::WorkCeilingDenial> {
    let product = guard
        .preparation_mut()
        .query
        .attempt
        .affinity()
        .product_publication()
        .clone();
    loop {
        context
            .checkpoint(1)
            .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
        let outcome = if matches!(guard.preparation_mut().world, World::Prepared(_)) {
            let World::Prepared(prepared) = take_world(guard, Call::Publication) else {
                unreachable!()
            };
            match prepared {
                QueryPreparedWorldPublication::Relational(prepared) => {
                    product.publication().execute_durable_without_signal(
                        execution,
                        prepared,
                        guard.preparation_mut().control.cancellation(),
                    )
                }
                QueryPreparedWorldPublication::WithSignal(prepared) => {
                    super::authored_definition::publish(execution, guard, &product, prepared)
                }
            }
        } else {
            let World::PendingPublication(pending) = take_world(guard, Call::PublicationAdvance)
            else {
                unreachable!()
            };
            pending.advance(&execution)
        };
        match outcome {
            WorldDurablePublicationOutcome::Performed(performed) => {
                guard.preparation_mut().world = World::Performed(performed);
                return Ok(Progress::Terminal);
            }
            WorldDurablePublicationOutcome::Pending(pending) => {
                guard.preparation_mut().world = World::PendingPublication(pending);
                guard.preparation_mut().stage = Stage::PublicationPending;
                std::thread::yield_now();
            }
            WorldDurablePublicationOutcome::ProductUnpublished(effects) => {
                guard.preparation_mut().world = World::Unpublished(effects);
                return Ok(Progress::Terminal);
            }
            WorldDurablePublicationOutcome::RecoveryRequired { cause, custody } => {
                guard.preparation_mut().world =
                    World::PublicationRecoveryRequired { cause, custody };
                guard.preparation_mut().stage = Stage::PublicationRecoveryRequired;
                return Ok(Progress::Pending(Stage::PublicationRecoveryRequired));
            }
        }
    }
}
