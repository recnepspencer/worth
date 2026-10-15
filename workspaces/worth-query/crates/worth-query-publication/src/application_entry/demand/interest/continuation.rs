//! Move-only custody of an admitted demand, independent of a caller's borrow.
use super::*;

/// The original admitted source, program selection and native interest.
/// It retains no principal or request scope. Drop releases the caller's interest;
/// every advance requires the original runtime and fresh authenticated authority.
#[must_use]
pub struct WorthQueryApplicationOutputDemandContinuation<Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub(in crate::application_entry) admitted:
        WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    pub(in crate::application_entry) controls: WorthQueryOutputDemandControls,
    pub(super) selected_program: Option<(ApplicationProgramIdentity, ApplicationProgramRevision)>,
    pub(super) closed: bool,
}

impl<Schema, Demand> WorthQueryApplicationOutputDemandContinuation<Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub(in crate::application_entry) fn new(
        admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
        controls: WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            admitted,
            controls,
            selected_program: None,
            closed: false,
        }
    }
    /// Borrows the pending native terminal; diagnostic denial copies cannot own it.
    pub fn managed_cleanup_failure(
        &mut self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.admitted.managed_cleanup_failure()
    }
    /// Releases actual failed producer cleanup under this runtime's fresh issuer.
    /// A release receipt is not output settlement; refusal keeps the native owner.
    pub fn release_managed_cleanup(
        &mut self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        Option<
            worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
        >,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if !std::ptr::eq(application, request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        application
            .with_application_advancement(request.scope, |phase| {
                self.admitted
                    .release_managed_cleanup(&phase)
                    .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?
    }
    pub fn close(&mut self) {
        if !self.closed {
            self.admitted.close();
            self.closed = true;
        }
    }
    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryApplicationOutputDemandDenial> {
        self.admitted
            .notifications()
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
    }
}

impl<Schema, Demand> Drop for WorthQueryApplicationOutputDemandContinuation<Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn drop(&mut self) {
        self.close();
    }
}
