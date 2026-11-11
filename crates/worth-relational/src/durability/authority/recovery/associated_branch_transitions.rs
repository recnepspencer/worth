//! Visit protected branch originals without replaying or admitting a live fork.
use super::super::RecoveredRelationalRuntimeAuthority;
use super::fork_effect::RelationalForkReadmissionCause as Denial;
use crate::{branch::RelationalBranchTransitionSequence, runtime::RelationalRuntime};
use std::ops::ControlFlow;
use worth_execution::ExecutionRequest;

impl RecoveredRelationalRuntimeAuthority {
    /// This inventory belongs to the exact recovered runtime. It grants no
    /// live currentness or absence proof; readmission's composition excludes
    /// ordinary publication. The callback runs outside Native's stream lock.
    pub fn inspect_recovered_associated_branch_transitions<B>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        mut visit: impl FnMut(&crate::branch::PositionedBranchTransition) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, Denial> {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        if self.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        let _operation = runtime
            .owner_binding()
            .admit()
            .ok_or(Denial::OwnerUnavailable)?;
        let routes = runtime.history.canonical_checkpoint_gate();
        let mut previous = RelationalBranchTransitionSequence(0);
        loop {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            let Some(original) = routes.next_associated_branch_transition(previous) else {
                break;
            };
            previous = original.sequence();
            if let ControlFlow::Break(value) = visit(&original) {
                return Ok(ControlFlow::Break(value));
            }
        }
        Ok(ControlFlow::Continue(()))
    }
}
