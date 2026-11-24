use super::{capture, CapturedNativeInterimAdmission};
use crate::durability::authority::runtime_access::DurabilityAuthority;
use worth_execution::ExecutionRequest;

impl DurabilityAuthority<'_> {
    pub fn capture_interim_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (
            crate::durability::data::RelationalNativeCheckpoint,
            CapturedNativeInterimAdmission,
        ),
        super::super::Denial,
    > {
        policy.check_live()?;
        crate::execution::check_request(request).map_err(super::super::Denial::Request)?;
        capture::capture(self.runtime, request)
    }
}
