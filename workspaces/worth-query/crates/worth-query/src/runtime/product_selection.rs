//! Issue the explicit Query token for the installed root product.

use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    /// Lends the installed owner's phase for one public host call.
    pub fn with_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
                'scope,
            >,
        ) -> R,
    ) -> Result<
        R,
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    > {
        self.runtime.installed_product.world.with_advancement(body)
    }

    /// Lends the installed phase and mutable workspace for one public host call.
    /// A nested advancement opening inside `body` returns `NestedOpening` before
    /// invoking its callback, exactly like the immutable opener.
    pub fn with_advancement_mut<R>(
        &mut self,
        body: impl for<'scope> FnOnce(
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
                'scope,
            >,
            &mut Self,
        ) -> R,
    ) -> Result<
        R,
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    > {
        let world = self.runtime.installed_product.world.clone();
        world.with_advancement(|phase| body(phase, self))
    }

    /// Opens one installed host call and lends its request to read seams.
    pub fn with_execution_request<R>(
        &self,
        body: impl for<'scope> FnOnce(&worth_execution::ExecutionRequest<'scope, 'scope>) -> R,
    ) -> Result<R, super::WorthQueryRuntimeError> {
        self.runtime
            .installed_product
            .world
            .with_advancement(|phase| {
                let execution = phase
                    .execution_request_for(&self.runtime.installed_product.world)
                    .map_err(super::WorthQueryRuntimeError::ForeignPhase)?;
                Ok(body(&execution))
            })
            .map_err(super::WorthQueryRuntimeError::Advancement)?
    }

    /// Opens one installed host call and lends its request to mutable workspace seams.
    pub fn with_execution_request_mut<R>(
        &mut self,
        body: impl for<'scope> FnOnce(
            &worth_execution::ExecutionRequest<'scope, 'scope>,
            &mut Self,
        ) -> R,
    ) -> Result<R, super::WorthQueryRuntimeError> {
        let world = self.runtime.installed_product.world.clone();
        world
            .with_advancement(|phase| {
                let execution = phase
                    .execution_request_for(&world)
                    .map_err(super::WorthQueryRuntimeError::ForeignPhase)?;
                Ok(body(&execution, self))
            })
            .map_err(super::WorthQueryRuntimeError::Advancement)?
    }

    pub fn current_world(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.runtime.installed_product.world.root_product_branch()
    }

    pub fn branches(
        &self,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranches<'_> {
        self.runtime.installed_product.world.product_branches()
    }
}

#[cfg(test)]
mod tests;
