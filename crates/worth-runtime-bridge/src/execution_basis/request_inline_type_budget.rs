//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeManagedExecutionIntent>(
            "crates/worth-runtime-bridge/src/execution_basis/request.rs",
            "BridgeManagedExecutionIntent",
            128 * 1024,
        );
        owner.field::<BridgeManagedExecutionIntentIdentity>("identity");
        owner.field::<Arc<str>>("operation_binding_identity");
        owner.field::<Arc<str>>("resource_attempt_identity");
    }
}
