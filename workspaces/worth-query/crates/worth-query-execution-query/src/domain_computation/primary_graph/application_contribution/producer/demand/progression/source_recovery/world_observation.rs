//! Original output basis DATA is resolved only by World's actual history owner.
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_execution::ExecutionRequest;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn readmit_output_observation(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        branch: &str,
    ) -> Result<worth_runtime_world::facade::ProductBranchObservation, Denial> {
        let world = self.runtime.product_runtime.owner_ref().observation_port();
        let request_stop = |cause| {
            Denial::new(
                Kind::ExecutionRequest(
                    crate::domain_computation::primary_graph::application_contribution::advancement_denial(
                        cause,
                    ),
                ),
                "required source readmission",
            )
        };
        let branches =
            world
                .installed_product_branches(execution)
                .map_err(|cause| match cause {
                    worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                        worth_runtime_world::facade::ProductBranchInventoryDenial::Request(cause),
                    ) => request_stop(cause),
                    _ => match execution.consult() {
                        Err(cause) => request_stop(cause),
                        Ok(()) => unavailable(),
                    },
                })?;
        let current = branches
            .branches()
            .iter()
            .find(|known| {
                known.branch_identity().name().as_str() == branch
                    && known.owner_identity().durable() == Some(commit.owner())
            })
            .ok_or_else(unavailable)?;
        if current.selected_commit().owner_identity().durable() == Some(commit.owner())
            && current.selected_commit().ordinal() == commit.ordinal()
            && let Ok(head) =
                world.observe_product_branch_occurrence(current.lifecycle_incarnation())
            && head.selected_commit().owner_identity().durable() == Some(commit.owner())
            && head.selected_commit().ordinal() == commit.ordinal()
        {
            return Ok(head);
        }
        let mut history = world
            .trace_product_branch_ancestry(execution, current.lifecycle_incarnation(), None)
            .map_err(|_| match execution.consult() {
                Err(cause) => request_stop(cause),
                Ok(()) => unavailable(),
            })?;
        let observation = loop {
            execution.consult().map_err(request_stop)?;
            if let Some(index) = history.commits().position(|entry| {
                entry.identity().owner_identity().durable() == Some(commit.owner())
                    && entry.identity().ordinal() == commit.ordinal()
            }) {
                break world
                    .observe_product_branch_history_entry(&history, index)
                    .map_err(|_| unavailable())?;
            }
            if history.is_complete() {
                return Err(unavailable());
            }
            history = world
                .continue_product_branch_ancestry(execution, &history, None)
                .map_err(|_| match execution.consult() {
                    Err(cause) => request_stop(cause),
                    Ok(()) => unavailable(),
                })?;
        };
        Ok(observation)
    }
}
fn unavailable() -> Denial {
    Denial::new(
        Kind::RetainedBasisUnavailable,
        "recorded output basis has no World original",
    )
}
