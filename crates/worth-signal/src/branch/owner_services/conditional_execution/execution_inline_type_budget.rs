//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalExecutionSlot>(
            "crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs",
            "SignalConditionalExecutionSlot",
            64,
        );
        owner
            .field::<Box<crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition>>(
                "partition",
            );
    }
    {
        let mut owner = census.owner::<SignalConditionalEvaluationState>(
            "crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs",
            "SignalConditionalEvaluationState",
            128 * 1024,
        );
        owner.field::<SignalConditionalRetentionReservation>("admission_custody");
        owner.field::<Option<SignalConditionalExecutionSlot>>("slot");
    }
    {
        let mut owner = census.owner::<SignalConditionalEvaluationAdmission>(
            "crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs",
            "SignalConditionalEvaluationAdmission",
            128 * 1024,
        );
        owner.field::<Arc<SignalConditionalServiceAuthority>>("service_authority");
        owner.field::<SignalConditionalServiceContractBinding>("contract_binding");
        owner.field::<InstalledSignalConditionalContract>("contract");
        owner.field::<Arc<SignalConditionalEvaluationSourceEvidence>>("source");
        owner.field::<Arc<str>>("execution_identity");
        owner.field::<Arc<SignalRetainedExecutionBasis>>("retained_basis");
        owner.field::<Mutex<SignalConditionalEvaluationState>>("execution");
    }
    {
        let _owner = census.owner::<SignalConditionalServiceExecutionDenial>(
            "crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs",
            "SignalConditionalServiceExecutionDenial",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<SignalConditionalServiceCompletion>(
            "crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs",
            "SignalConditionalServiceCompletion",
            128 * 1024,
        );
        owner
            .field::<Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>>(
                "decision",
            );
        owner.field::<Result<Option<SignalInvalidationExecutionReceipt>, SignalError>>(
            "observation",
        );
        owner.field::<SignalConditionalEvaluationBindingEvidence>("binding");
        owner.field::<bool>("slot_reused");
    }
}
