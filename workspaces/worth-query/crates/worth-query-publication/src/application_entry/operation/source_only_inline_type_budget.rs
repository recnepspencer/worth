//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQuerySourceCompletion<()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs", "WorthQuerySourceCompletion", 128 * 1024);
        owner.field::<WorthQueryApplicationCommitReceipt>("Fresh.receipt");
        owner.field::<()>("Fresh.result");
        owner.field::<Option<WorthQueryApplicationReadObservation>>("Fresh.retained");
        owner.field::<Option<WorthQueryApplicationReadObservation>>("Recovered.retained");
        owner.field::<WorthQueryApplicationCommitReceipt>("Recovered.receipt");
        owner.field::<Option<WorthQueryManagedApplicationRecoveryPerformed>>(
            "Recovered.publication",
        );
        owner.field::<Option<WorthQueryApplicationUncommitted>>("Recovered.initial_posture");
        owner.field::<Option<
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
        >>("Recovered.initial_advancement_failure");
        owner.field::<Option<WorthQueryApplicationReadObservation>>("Recorded.retained");
        owner.field::<worth_query_execution::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt>("Recorded.receipt");
        owner
            .field::<Option<WorthQueryManagedApplicationRecoveryPerformed>>("Recorded.publication");
        owner.field::<Option<WorthQueryApplicationUncommitted>>("Recorded.initial_posture");
        owner.field::<Option<
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
        >>("Recorded.initial_advancement_failure");
    }
    {
        let mut owner = census.owner::<WorthQuerySourceOnlyAdvance<(), (), ()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs", "WorthQuerySourceOnlyAdvance", 128 * 1024);
        owner.field::<()>("Refused.operation");
        owner.field::<WorthQuerySourceOnlyDenial>("Refused.denial");
    }
    {
        let _owner = census.owner::<WorthQuerySourceOnlyDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs", "WorthQuerySourceOnlyDenial", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQuerySourceOnlyPosture>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/source_only.rs", "WorthQuerySourceOnlyPosture", 128 * 1024);
    }
}
