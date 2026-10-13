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
    pub(super) admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    pub(super) controls: WorthQueryOutputDemandControls,
    pub(super) selected_program: Option<(ApplicationProgramIdentity, ApplicationProgramRevision)>,
    pub(super) closed: bool,
}

impl<Schema, Demand> WorthQueryApplicationOutputDemandContinuation<Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub(super) fn new(
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
