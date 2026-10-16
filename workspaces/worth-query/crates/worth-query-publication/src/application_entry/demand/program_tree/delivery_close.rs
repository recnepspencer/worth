//! Positive full-tree custody remains close-only through any discharged prefix.
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
    pub fn retry_close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedProgramOutputDelivery<S, P, R>, Self> {
        close_for_delivery(self.custody, self.delivery, request)
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
    match custody.close_native(&delivery.runtime, delivery.branch, request) {
        Ok(closed) => {
            // Retire original demand/controls/extra basis and empty native carriers
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
