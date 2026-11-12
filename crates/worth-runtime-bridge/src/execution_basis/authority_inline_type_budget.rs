//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeBoundExecutionBasis>(
            "crates/worth-runtime-bridge/src/execution_basis/authority.rs",
            "BridgeBoundExecutionBasis",
            128 * 1024,
        );
        owner.field::<BridgeExecutionBasisIdentity>("identity");
        owner.field::<u64>("bridge_runtime_key");
        owner.field::<BridgeManagedExecutionIntent>("managed_intent");
        owner.field::<BridgeExecutionBasisPosture>("posture");
        owner.field::<AdmittedBridgeAsyncRequestIdentity>("request");
        owner.field::<Option<MaterializedTruthViewObservation>>("observation");
        owner.field::<Option<BridgeAuthoritativeSourceProfile>>("authoritative_source_profile");
        owner.field::<Option<BridgeExecutionBasisReservation>>("reservation");
        owner.field::<bool>("signal_terminalized");
        owner.field::<BridgeExecutionBasisCounters>("counters");
    }
    {
        let mut owner = census.owner::<BridgeBoundExecutionBasisParts>(
            "crates/worth-runtime-bridge/src/execution_basis/authority.rs",
            "BridgeBoundExecutionBasisParts",
            128 * 1024,
        );
        owner.field::<u64>("bridge_runtime_key");
        owner.field::<BridgeManagedExecutionIntent>("managed_intent");
        owner.field::<BridgeExecutionBasisPosture>("posture");
        owner.field::<AdmittedBridgeAsyncRequestIdentity>("request");
        owner.field::<MaterializedTruthViewObservation>("observation");
        owner.field::<Option<BridgeAuthoritativeSourceProfile>>("authoritative_source_profile");
        owner.field::<BridgeExecutionBasisReservation>("reservation");
        owner.field::<BridgeExecutionBasisCounters>("counters");
    }
}
