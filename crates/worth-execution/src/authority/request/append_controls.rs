//! Immediate cancellation for scalar moves between bounded complete consultations.
use super::{ExecutionRequest, RequestBacking};
use crate::WorkCeilingDenial;
impl ExecutionRequest<'_, '_> {
    pub(in crate::authority) fn consult_append_cancellation(self) -> Result<(), WorkCeilingDenial> {
        let own = match self.form {
            RequestBacking::Leased(lease) => lease.is_cancelled(),
            RequestBacking::Serial(request) => request.cancellation.is_cancelled(),
        };
        if own || crate::backend::active_cancelled() {
            self.consult()
        } else {
            Ok(())
        }
    }
}
