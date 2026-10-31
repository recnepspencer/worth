//! One pre-effect preparation on the supplied request's actual execution meter.
use super::{ExecutionRequest, RequestBacking};
use crate::{ExecutionResourceLease, ExecutionWorkCeiling, WorkCeilingDenial};

impl<'authority> ExecutionRequest<'_, 'authority> {
    /// Prepare values before effects using one original request scope.
    ///
    /// Already-active leased carriage reuses its actual descendant and meter.
    /// Standalone entry, or a supplied child below an active ancestor, enters
    /// the existing native scope on that exact lease. Foreign and sibling
    /// leases retain normal admission refusal. Serial controls remain on their
    /// original backing. No request or additional allowance is created.
    ///
    /// Success passes final kernel controls; a stop disposes prepared values.
    /// This operation must not publish effects or return changed runtime custody
    /// that would need to survive a failed final handoff. Domain failures retain
    /// their typed cause and do not become successful prepared values.
    pub fn prepare_native<R, E>(
        self,
        operation: impl for<'scope> FnOnce(
            Option<&'scope ExecutionResourceLease<'authority>>,
        ) -> Result<R, E>,
    ) -> Result<Result<R, E>, WorkCeilingDenial> {
        if let RequestBacking::Leased(lease) = self.form
            && let Some(active) = lease.current_descendant()
            && crate::backend::has_active_kernel()
        {
            return crate::backend::run_invoking_preparation(|| operation(Some(&active)));
        }
        self.run_native_report(ExecutionWorkCeiling::new(None), operation)
            .map(|(result, _)| result)
    }
}
