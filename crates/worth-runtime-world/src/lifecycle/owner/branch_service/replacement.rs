//! Paired lifecycle transition with explicit post-effect handoff custody.
use super::super::RuntimeWorldOwnerRoot;
use crate::branch::{
    ProductBranchObservation, RuntimeWorldBranchReplacementDenial as Denial,
    RuntimeWorldRetiredReplacement,
};
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};

pub(super) fn retire<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    execution: ExecutionRequest<'_, '_>,
    old: &ProductBranchObservation,
    candidate: &ProductBranchObservation,
) -> Result<RuntimeWorldRetiredReplacement, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if old.owner_identity() != owner.owner_identity()
        || candidate.owner_identity() != owner.owner_identity()
    {
        return Err(Denial::ForeignOwner);
    }
    if !owner.branch_service_is_available() {
        return Err(Denial::OwnerUnavailable);
    }
    let _operation = owner
        .reserve_creation_operation()
        .map_err(|()| Denial::OwnerUnavailable)?;
    // The saved terminal value outlives the SDK callback. An original-request
    // stop at its handoff cannot erase retirement that already linearized.
    let mut terminal = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        let prepared = owner.state.custody.with_retirement_work(
            old.branch_identity(),
            old.lifecycle_incarnation(),
            execution,
            context,
            |context| {
                owner
                    .state
                    .branches
                    .retire_replaced_head(old, candidate, execution, context)
            },
        );
        match prepared {
            Ok(((cell, retirement, boundary), work)) => {
                let (retired_head, protection) = retirement.into_parts();
                terminal = Some(Ok(RuntimeWorldRetiredReplacement {
                    incarnation: old.lifecycle_incarnation(),
                    retired_head,
                    boundary,
                    handoff_stop: None,
                    work,
                }));
                // Custody, registry and both reference guards have all dropped.
                // Save the genuine effect before any owner destructor can
                // stop or panic, then release its old component/history pins.
                drop(cell);
                drop(protection);
            }
            Err(cause) => terminal = Some(Err(cause)),
        }
        #[cfg(test)]
        if terminal.as_ref().is_some_and(Result::is_ok) {
            testing::after_retirement(owner.owner_identity());
        }
    });
    match (terminal, outcome) {
        (Some(Ok(mut retired)), outcome) => {
            retired.handoff_stop = outcome.err();
            Ok(retired)
        }
        (Some(Err(cause)), Ok(_))
        | (Some(Err(cause)), Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))) => {
            Err(cause)
        }
        (_, Err(cause)) => Err(Denial::Request(cause)),
        (None, Ok(_)) => unreachable!("completed paired retirement callback"),
    }
}

#[cfg(test)]
pub(super) mod testing;
