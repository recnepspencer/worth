//! Runtime identity is checked before lineage admission touches Signal state.
use super::rejection::{
    BridgeAsyncForwardCausalityRejection, BridgeAsyncForwardCausalityRejectionKind,
};
use crate::source::AdmittedBridgeAsyncRequestIdentity;

pub(crate) fn validate_lineage_runtime(
    runtime_key: u64,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> Result<(), BridgeAsyncForwardCausalityRejection> {
    if request.bridge_runtime_key() == runtime_key {
        Ok(())
    } else {
        Err(runtime_mismatch())
    }
}

pub(super) fn runtime_mismatch() -> BridgeAsyncForwardCausalityRejection {
    BridgeAsyncForwardCausalityRejection::new(
        BridgeAsyncForwardCausalityRejectionKind::SignalRuntimeIdentityMismatch,
        "forward causality requires prior and newer requests from the selected Bridge runtime",
    )
}
