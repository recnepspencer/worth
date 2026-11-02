//! Close is original-affine cleanup; selected roots and current sources are irrelevant.
//!
//! A runtime phase can dispose of an owner and retry its close-only remainder:
//! ```
//! use worth_query_publication::facade::application_entry::{WorthQueryProgramOutputOperation, WorthQueryProgramOutputCloseRefusal};
//! use worth_query_execution::facade::{application_contribution::WorthQueryAdvancementPhase, primary_graph::WorthQueryApplicationRequiredOutputConnection};
//! use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
//! fn dispose<S,P,R>(
//!     owner: WorthQueryProgramOutputOperation<S,P,R>,
//!     remainder: WorthQueryProgramOutputCloseRefusal<S,P,R>,
//!     phase: &WorthQueryAdvancementPhase<'_>,
//! ) where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _owner_result = owner.close_in_advancement(phase);
//!     let _remainder_result = remainder.retry_close_in_advancement(phase);
//! }
//! ```
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
    pub(super) fn from_cleanup(cleanup: Option<WorthQueryOutputReadCleanup>) -> Self {
        Self {
            cleanup,
            terminal: None,
            construction: None,
        }
    }
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
    denial: WorthQueryProgramOutputCloseDenial,
    // Runtime follows every original native field, including the denial.
    core: Core<S, P, R>,
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
            .custody
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

    /// Retries only original disposal under a live phase from the owning runtime.
    pub fn retry_close_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, Self> {
        close_in_advancement(
            self.core,
            self.settlement,
            self.terminal,
            self.construction,
            phase,
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
    let result = core
        .custody
        .close_native(&core.runtime, core.branch, request);
    finish_close(core, settlement, terminal, construction, result)
}
fn close_in_advancement<S, P, R>(
    mut core: Core<S, P, R>,
    settlement: Option<WorthQueryApplicationProgramOutputSettlement<Query<S, R>>>,
    terminal: Option<WorthQueryProgramOutputAdvanceDenial>,
    construction: Option<WorthQueryProgramOutputConstructionDenial>,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    let result = core
        .custody
        .close_native_in_advancement(&core.runtime, core.branch, phase);
    finish_close(core, settlement, terminal, construction, result)
}
fn finish_close<S, P, R>(
    core: Core<S, P, R>,
    settlement: Option<WorthQueryApplicationProgramOutputSettlement<Query<S, R>>>,
    terminal: Option<WorthQueryProgramOutputAdvanceDenial>,
    construction: Option<WorthQueryProgramOutputConstructionDenial>,
    result: Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseDenial>,
) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    match result {
        Ok(mut closed) => {
            closed.terminal = terminal;
            closed.construction = construction;
            Ok(closed)
        }
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

    /// Discharges this original request without admitting a new application read.
    pub fn close_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close_in_advancement(self.core, None, None, None, phase)
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
        let ProgramOutputDeliveryCore {
            settlement,
            source,
            branch,
            runtime,
        } = self.delivery;
        drop(source);
        close(
            Core {
                custody: self.custody,
                branch,
                runtime,
            },
            Some(settlement),
            None,
            None,
            request,
        )
    }

    /// Discharges native interests and drops the original readonly delivery pins.
    pub fn close_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        let ProgramOutputDeliveryCore {
            settlement,
            source,
            branch,
            runtime,
        } = self.delivery;
        drop(source);
        close_in_advancement(
            Core {
                custody: self.custody,
                branch,
                runtime,
            },
            Some(settlement),
            None,
            None,
            phase,
        )
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

    /// Retains the terminal cause while discharging the original native remainder.
    pub fn close_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close_in_advancement(self.core, None, Some(self.denial), None, phase)
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

    /// Discharges original construction custody without a replacement request.
    pub fn close_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseRefusal<S, P, R>>
    {
        close_in_advancement(self.core, None, None, Some(self.denial), phase)
    }
}
