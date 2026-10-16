//! Close is original-affine cleanup; selected roots and current sources are irrelevant.
use super::*;
/// Why native disposal refused; the accompanying owner retains remaining custody.
#[derive(Debug)]
pub enum WorthQueryProgramOutputCloseDenial {
    FreshRequestMismatch,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Native(WorthQueryOutputReadCloseDenial),
}
/// Original read request custody is discharged; this is no positive output proof.
pub struct WorthQueryClosedProgramOutputRequest {
    cleanup: Option<WorthQueryOutputReadCleanup>,
    terminal: Option<WorthQueryProgramOutputAdvanceDenial>,
    construction: Option<WorthQueryProgramOutputConstructionDenial>,
}
impl WorthQueryClosedProgramOutputRequest {
    pub fn retired_component_count(&mut self) -> usize {
        self.cleanup
            .as_mut()
            .map_or(0, WorthQueryOutputReadCleanup::retired_component_count)
    }

    pub fn terminal_denial(&self) -> Option<&WorthQueryProgramOutputAdvanceDenial> {
        self.terminal.as_ref()
    }
    pub fn construction_denial(&self) -> Option<&WorthQueryProgramOutputConstructionDenial> {
        self.construction.as_ref()
    }
    pub fn managed_cleanup_receipt(
        &mut self,
    ) -> Option<&worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt>
    {
        self.cleanup
            .as_mut()
            .and_then(WorthQueryOutputReadCleanup::receipt)
    }
}
/// An interrupted close owns the entire original context and its released prefix.
/// Retry close only: released native interests cannot be advanced again.
#[must_use]
pub struct WorthQueryProgramOutputCloseRefusal<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    settlement: Option<WorthQueryApplicationProgramOutputSettlement<Query<S, R>>>,
    terminal: Option<WorthQueryProgramOutputAdvanceDenial>,
    construction: Option<WorthQueryProgramOutputConstructionDenial>,
    core: Core<S, P, R>,
    denial: WorthQueryProgramOutputCloseDenial,
}
impl<S, P, R> WorthQueryProgramOutputCloseRefusal<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn denial(&self) -> &WorthQueryProgramOutputCloseDenial {
        &self.denial
    }
    pub fn construction_denial(&self) -> Option<&WorthQueryProgramOutputConstructionDenial> {
        self.construction.as_ref()
    }
    pub fn producer_outcome(
        &mut self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted>
    {
        self.core
            .cleanup
            .as_mut()
            .and_then(WorthQueryOutputReadCleanup::producer_outcome)
    }
    pub fn terminal_denial(&self) -> Option<&WorthQueryProgramOutputAdvanceDenial> {
        self.terminal.as_ref()
    }
    pub fn settlement(&self) -> Option<&WorthQueryApplicationProgramOutputSettlement<Query<S, R>>> {
        self.settlement.as_ref()
    }
}
impl<S, P, R> WorthQueryProgramOutputCloseRefusal<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn retry_close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, Self> {
        close(
            self.core,
            self.settlement,
            self.terminal,
            self.construction,
            request,
        )
    }
}
fn close<S, P, R>(
    mut core: Core<S, P, R>,
    settlement: Option<WorthQueryApplicationProgramOutputSettlement<Query<S, R>>>,
    terminal: Option<WorthQueryProgramOutputAdvanceDenial>,
    construction: Option<WorthQueryProgramOutputConstructionDenial>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    use WorthQueryProgramOutputCloseDenial as Denial;
    let result = if !core.matches(request) {
        Err(Denial::FreshRequestMismatch)
    } else {
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                phase
                    .checked_execution_request_for(request.application.product_runtime())
                    .map_err(Denial::Execution)?;
                if let Some(cleanup) = core.cleanup.as_mut() {
                    cleanup
                        .close_in_advancement(&phase, request.application)
                        .map_err(Denial::Native)?;
                }
                if let Some(tree) = core.tree.as_mut() {
                    tree.close_read_request(&phase, &core.runtime, core.branch)
                        .map_err(Denial::Native)?;
                }
                Ok(())
            })
            .map_err(Denial::Execution)
            .and_then(|result| result)
    };
    match result {
        Ok(()) => Ok(WorthQueryClosedProgramOutputRequest {
            cleanup: core.cleanup.take(),
            terminal,
            construction,
        }),
        Err(denial) => Err(WorthQueryProgramOutputCloseRefusal {
            core,
            settlement,
            terminal,
            construction,
            denial,
        }),
    }
}
impl<S, P, R> WorthQueryProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close(self.core, None, None, None, request)
    }
}
impl<S, P, R> WorthQuerySettledProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close(self.core, Some(self.settlement), None, None, request)
    }
}
impl<S, P, R> WorthQueryTerminalProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close(self.core, None, Some(self.denial), None, request)
    }
}
impl<S, P, R> WorthQueryProgramOutputConstructionRefusal<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close(self.core, None, None, Some(self.denial), request)
    }
}
