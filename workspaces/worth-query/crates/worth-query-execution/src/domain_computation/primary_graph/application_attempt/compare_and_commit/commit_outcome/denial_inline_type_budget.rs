//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationCommitDenialKind>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_outcome/denial.rs", "WorthQueryApplicationCommitDenialKind", 128 * 1024);
        owner.field::<crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial>("ExecutionResource.denial");
        owner.field::<Option<u64>>("ExecutionResource.partition_identity");
        owner.field::<Option<u32>>("ExecutionResource.policy_ancestor");
        owner.field::<Option<u64>>("ExecutionNestedPatternStopped.partition_identity");
        owner.field::<Option<u64>>("ExecutionWorkerPanicked.partition_identity");
        owner.field::<Option<u64>>("ExecutionUncheckedCustomKernel.partition_identity");
        owner.field::<Option<u64>>("ExecutionIdentitiesNotCanonical.partition_identity");
        owner.field::<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind>("WorkflowSettlementDenied.kind");
        owner.field::<usize>("ActiveSnapshotCapacityExhausted.maximum_active_snapshots");
        owner.field::<worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial>("SignedCreationAuthentication.kind");
        owner.field::<usize>("SigningLifecycleEvidenceCapacity.maximum_evidence");
        owner.field::<usize>("SigningLifecycleSubjectCapacity.maximum_subjects");
        owner.field::<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind>("SigningLifecycleObservation.kind");
        owner.field::<worth_query_declaration::facade::application_program::ApplicationProgramRevision>("ProgramNotActiveOnOccurrence.active");
    }
    {
        let mut owner = census.owner::<WorthQueryApplicationCommitDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_outcome/denial.rs", "WorthQueryApplicationCommitDenial", 128 * 1024);
        owner.field::<WorthQueryApplicationCommitDenialKind>("kind");
        owner.field::<WorthQueryApplicationCommitDenialStage>("stage");
        owner.field::<Option<std::sync::Arc<str>>>("detail");
        owner.field::<Option<Box<denial_cause::DenialCause>>>("cause");
    }
}
