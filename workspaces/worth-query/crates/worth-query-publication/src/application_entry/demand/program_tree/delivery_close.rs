//! Positive full-tree custody remains close-only through any discharged prefix.
//!
//! Closing inside a phase preserves the original positive delivery context:
//! ```
//! use worth_query_publication::facade::application_entry::{WorthQuerySettledProgramOutputOperation, WorthQueryProgramOutputDeliveryCloseRefusal};
//! use worth_query_execution::facade::{application_contribution::WorthQueryAdvancementPhase, primary_graph::WorthQueryApplicationRequiredOutputConnection};
//! use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
//! fn dispose<S,P,R>(
//!     owner: WorthQuerySettledProgramOutputOperation<S,P,R>,
//!     remainder: WorthQueryProgramOutputDeliveryCloseRefusal<S,P,R>,
//!     phase: &WorthQueryAdvancementPhase<'_>,
//! ) where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _owner_result = owner.close_for_delivery_in_advancement(phase);
//!     let _remainder_result = remainder.retry_close_in_advancement(phase);
//! }
//! ```
use super::*;
/// Refused delivery-close owns the original positive settlement and native remainder.
/// Only fresh close retry is legal: a released prefix cannot return to live delivery.
#[must_use]
pub struct WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    denial: WorthQueryProgramOutputCloseDenial,
    custody: ReadCustody<S, P, R>,
    // Complete positive settlement and original runtime; never Option or recreated.
    delivery: ProgramOutputDeliveryCore<S, P, R>,
}
impl<S, P, R> WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn denial(&self) -> &WorthQueryProgramOutputCloseDenial {
        &self.denial
    }
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.delivery.branch
    }
    /// Retries only original native cleanup under fresh allocation/branch-affine authority.
    /// Any released prefix stays discharged; source/program supersession does not
    /// require a new demand or permit this owner to return to live delivery.
    pub fn retry_close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputDelivery<S, P, R>, Self> {
        close_for_delivery(self.custody, self.delivery, request)
    }

    /// Retries the same native remainder under the owning runtime's live phase.
    pub fn retry_close_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryClosedProgramOutputDelivery<S, P, R>, Self> {
        close_for_delivery_in_advancement(self.custody, self.delivery, phase)
    }
}
impl<S, P, R> WorthQuerySettledProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    /// Discharges ALL original native read interests and cleanup before delivery.
    /// Close uses original allocation/branch and fresh authority, even if superseded.
    /// A refusal retains a close-only owner including any already-discharged prefix.
    pub fn close_for_delivery(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryClosedProgramOutputDelivery<S, P, R>,
        WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>,
    > {
        close_for_delivery(self.custody, self.delivery, request)
    }

    /// Closes native interests while preserving the original readonly delivery.
    /// A live phase from the owning runtime suffices; no application READ is made.
    pub fn close_for_delivery_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryClosedProgramOutputDelivery<S, P, R>,
        WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>,
    > {
        close_for_delivery_in_advancement(self.custody, self.delivery, phase)
    }
}
fn close_for_delivery<S, P, R>(
    mut custody: ReadCustody<S, P, R>,
    delivery: ProgramOutputDeliveryCore<S, P, R>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> Result<
    WorthQueryClosedProgramOutputDelivery<S, P, R>,
    WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>,
>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    let result = custody.close_native(&delivery.runtime, delivery.branch, request);
    finish_close(custody, delivery, result)
}
fn close_for_delivery_in_advancement<S, P, R>(
    mut custody: ReadCustody<S, P, R>,
    delivery: ProgramOutputDeliveryCore<S, P, R>,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> Result<
    WorthQueryClosedProgramOutputDelivery<S, P, R>,
    WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>,
>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    let result = custody.close_native_in_advancement(&delivery.runtime, delivery.branch, phase);
    finish_close(custody, delivery, result)
}
fn finish_close<S, P, R>(
    custody: ReadCustody<S, P, R>,
    delivery: ProgramOutputDeliveryCore<S, P, R>,
    result: Result<WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseDenial>,
) -> Result<
    WorthQueryClosedProgramOutputDelivery<S, P, R>,
    WorthQueryProgramOutputDeliveryCloseRefusal<S, P, R>,
>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    match result {
        Ok(closed) => {
            // Retire original demand/controls and empty native carriers
            // while this SAME runtime is still alive in the delivery core.
            drop(custody);
            Ok(WorthQueryClosedProgramOutputDelivery { closed, delivery })
        }
        Err(denial) => Err(WorthQueryProgramOutputDeliveryCloseRefusal {
            denial,
            custody,
            delivery,
        }),
    }
}
