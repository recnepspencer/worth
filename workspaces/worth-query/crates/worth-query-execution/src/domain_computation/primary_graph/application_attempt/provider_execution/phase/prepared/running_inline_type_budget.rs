//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryRunningApplicationCommit<(), (), (), ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared/running.rs", "WorthQueryRunningApplicationCommit", 128 * 1024);
        owner.field::<completion_storage::ApplicationCommitCompletionStorage>("completion_storage");
        owner.field::<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        (),
        (),
        (),
        (),
    >>("admission");
        owner.field::<WorthQueryApplicationSnapshotLease>("lease");
        owner.field::<WorthQueryPreparedApplicationProviderAttempt>("provider_attempt");
        owner.field::<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity>("outcome_identity");
        owner
            .field::<Option<LocalWorkflowSettlementPublication>>("workflow_settlement_publication");
        owner.field::<crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization>("authorization");
        owner.field::<WorthQueryApplicationIdempotencyBinding>("idempotency");
        owner.field::<crate::domain_computation::WorthQueryRunningDirectRun>("running");
        owner.field::<crate::domain_computation::provider_session::WorthQueryMutationRunBinding>(
            "mutation_run",
        );
        owner.field::<WorthQueryApplicationAttemptBasis>("attempt_basis");
        owner.field::<Option<
            crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >>("aftermath_causality");
    }
}
