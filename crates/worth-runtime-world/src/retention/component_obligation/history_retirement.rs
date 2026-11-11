//! Original component leases and their inert retired allocations.
use crate::lifecycle::RuntimeWorldCloseDenial as Denial;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchRetentionLease,
    change_source::{
        RelationalRetainedObservationRetirement,
        RelationalRetainedObservationRetirementProgress as NativeProgress,
    },
};
use worth_signal::facade::branch::{
    SignalBranchRetentionLease, SignalBranchRetentionRetirement,
    SignalBranchRetentionRetirementDenial as SignalDenial,
};

#[derive(Debug)]
pub(crate) struct RetiringHistoryComponents {
    native: Option<RelationalRetainedObservationRetirement>,
    signal: Option<SignalBranchRetentionRetirement>,
}
impl RetiringHistoryComponents {
    pub(crate) fn new(
        native: Option<RelationalBranchRetentionLease>,
        signal: Option<SignalBranchRetentionLease>,
    ) -> Self {
        Self {
            native: native.map(|v| v.into_observation_retirement()),
            signal: signal.map(|v| v.into_retirement()),
        }
    }
    pub(crate) fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Denial> {
        if let Some(native) = &mut self.native {
            match native.advance(request) {
                NativeProgress::Advanced => return Ok(false),
                NativeProgress::Busy => return Err(Denial::Busy),
                NativeProgress::Stopped(cause) => return Err(Denial::Execution(cause)),
                NativeProgress::Refused(cause) => {
                    return Err(Denial::RecoveredNativeRetirement(cause))
                }
                NativeProgress::Complete => {}
            }
        }
        if let Some(signal) = &mut self.signal {
            signal.advance(request).map_err(|cause| match cause {
                SignalDenial::Busy => Denial::Busy,
                SignalDenial::Execution(cause) => Denial::Execution(cause),
            })?;
        }
        Ok(true)
    }
}
