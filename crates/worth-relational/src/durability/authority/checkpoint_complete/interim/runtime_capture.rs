use super::super::Denial;
use super::{capture::encode_capture, verified::freeze::freeze, CapturedNativeInterimAdmission};
use crate::durability::authority::runtime_access::DurabilityAuthority;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::runtime::RelationalRuntime;
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
        capture(self.runtime, request)
    }
}

fn capture(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalNativeCheckpoint, CapturedNativeInterimAdmission), Denial> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| {
        answer = Some(freeze(runtime, request).and_then(|frozen| encode_capture(frozen, request)));
    });
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the capture stage produced its answer"),
        (Err(cause), Some(Err(inner)))
            if crate::execution::RelationalExecutionDenialCause::from(cause)
                == crate::execution::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(Denial::Request(cause.into())),
    }
}
