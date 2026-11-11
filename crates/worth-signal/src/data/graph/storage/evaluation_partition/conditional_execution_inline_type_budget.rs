//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<SignalPartitionConditionalDenial>("crates/worth-signal/src/data/graph/storage/evaluation_partition/conditional_execution.rs", "SignalPartitionConditionalDenial", 128 * 1024);
    }
    {
        let mut owner = census.owner::<SignalPartitionConditionalCompletion>("crates/worth-signal/src/data/graph/storage/evaluation_partition/conditional_execution.rs", "SignalPartitionConditionalCompletion", 128 * 1024);
        owner
            .field::<Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>>(
                "decision",
            );
        owner.field::<Result<Option<SignalInvalidationExecutionReceipt>, SignalError>>(
            "observation",
        );
        owner.field::<Option<SignalRejectedConditionalEvaluation>>("rejected");
    }
    {
        let mut owner = census.owner::<ActivatedConditionalOutcome>("crates/worth-signal/src/data/graph/storage/evaluation_partition/conditional_execution.rs", "ActivatedConditionalOutcome", 128 * 1024);
        owner.field::<Box<dyn std::any::Any + Send>>("Unwound.payload");
        owner.field::<SignalPartitionConditionalUnwindReason>("Unwound.report");
    }
}
