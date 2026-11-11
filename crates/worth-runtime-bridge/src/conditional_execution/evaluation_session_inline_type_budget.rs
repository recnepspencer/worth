//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeConditionalEvaluationSource<'static>>(
            "crates/worth-runtime-bridge/src/conditional_execution/evaluation_session.rs",
            "BridgeConditionalEvaluationSource",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<BridgeConditionalEvaluationAdmissionRequest<'static>>(
            "crates/worth-runtime-bridge/src/conditional_execution/evaluation_session.rs",
            "BridgeConditionalEvaluationAdmissionRequest",
            128 * 1024,
        );
        owner.field::<&'static Arc<BridgeInstalledConditionalLowering>>("lowering");
        owner.field::<BridgeConditionalEvaluationSource<'static>>("source");
        owner.field::<&'static super::super::BridgeConditionalSignalBasisBinding>("signal_basis");
    }
    {
        let mut owner = census.owner::<BridgeConditionalEvaluationSession>(
            "crates/worth-runtime-bridge/src/conditional_execution/evaluation_session.rs",
            "BridgeConditionalEvaluationSession",
            128 * 1024,
        );
        owner.field::<u64>("bridge_runtime_key");
        owner.field::<Arc<BridgeInstalledConditionalLowering>>("lowering");
        owner.field::<Option<Arc<BridgeAdmittedSnapshot>>>("source_snapshot");
        owner.field::<SignalConditionalEvaluationAdmission>("signal");
        owner.field::<super::super::signal_port::BridgeConditionalSignalPort>("signal_port");
        owner.field::<usize>("snapshot_admission_attempts");
        owner.field::<Option<crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts>>("managed_source_record");
        owner.field::<Arc<super::super::observation_retention::BridgeObservationBaselines>>(
            "observation_baselines",
        );
        owner.field::<Option<super::super::BridgeConditionalEvaluationReadmissionCounters>>(
            "readmission_counters",
        );
    }
}
