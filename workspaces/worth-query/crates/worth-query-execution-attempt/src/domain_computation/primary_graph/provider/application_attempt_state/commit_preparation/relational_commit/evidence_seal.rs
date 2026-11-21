//! Exact-commit evidence minted only from the committed session stage.

pub mod postcommit_currentness;
pub use postcommit_currentness::{FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason};
pub use postcommit_currentness::{FailedRebase, RebaseVerificationFailure};

use super::commit_execution::WorthQueryCommittedApplicationSession;
use crate::domain_computation::primary_graph::provider::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters,
    session_commit::{
        WorthQueryCommittedDispatchOutboxResolution, WorthQueryPreImageRetentionWork,
    },
};

pub use worth_query_execution_lineage::facade::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::WorthQueryPrimaryGraphCommitEvidence;

pub struct WorthQueryMutationWorkCommitSeal {
    counters: WorthQueryPrimaryMutationWorkCounters,
    index_maintenance_work: worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
    touched_records: std::sync::Arc<
        worth_execution::ExecutionArray<
            crate::domain_computation::primary_graph::provider::WorthQueryTouchedRecordIdentity,
        >,
    >,
    preimage: WorthQueryPreImageRetentionWork,
}

pub(super) fn seal(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    committed: &mut WorthQueryCommittedApplicationSession,
) -> WorthQueryPrimaryGraphCommitEvidence {
    provider.graph.with_runtime(execution, |runtime| {
        seal_on_runtime(execution, runtime, committed)
    })
}
pub(super) fn seal_on_runtime(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    committed: &mut WorthQueryCommittedApplicationSession,
) -> WorthQueryPrimaryGraphCommitEvidence {
    let prepared_touched_records = committed.take_prepared_touched_records();
    let touched_records = prepared_touched_records.verify_performed(committed.changed_records());
    let mutation_work = crate::domain_computation::primary_graph::provider::mutation_work::mutation_work_from_commit_seal(
        WorthQueryMutationWorkCommitSeal {
            counters: committed.work(),
            index_maintenance_work: committed.index_maintenance_work(),
            touched_records,
            preimage: committed.preimage_retention_work(),
        },
    );
    let committed_dispatch_outbox = WorthQueryCommittedDispatchOutboxResolution::from_commit(
        committed.attempt().dispatch_outbox(),
        committed.committed(),
    );
    let output_correspondence = committed.output_correspondence();
    let prepared_rebase = committed.take_source_fact_rebase();
    let producer_output = committed
        .attempt()
        .idempotency()
        .producer_dependency_identity()
        .is_some();
    let performed_input_facts = producer_output
        .then(|| prepared_rebase.performed_input_facts())
        .flatten();
    let retain_output_field_evidence = prepared_rebase.retains_output_field_evidence();
    let request = committed.attempt().affinity().publication_request().clone();
    let committed_result = committed.committed();
    let observed_source_facts = {
        postcommit_currentness::rebase_output(
            execution,
            runtime,
            &committed_result.snapshot,
            prepared_rebase,
            &output_correspondence,
            committed.changed_records(),
            committed.coupled_commit(),
            producer_output,
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), Some(&request)),
        )
    };
    // Opt-in output evaluation keeps only immutable field evidence. A failed
    // optional capture cannot undo the already committed Native publication.
    let performed_output_fields = if retain_output_field_evidence {
        use crate::domain_computation::primary_graph::application_attempt::OutputFieldCaptureFailure as Failure;
        Some(match observed_source_facts.retained() {
            Err(cause) => Err(Failure::Rebase { reason: cause.reason.into_reason(), own_effect: cause.own_effect }),
            Ok(facts) => crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload::copy_performed_field_reads(
                &facts,
                crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), Some(&request)),
            ).map_err(Failure::Retention),
        })
    } else {
        None
    };
    WorthQueryPrimaryGraphCommitEvidence {
        workflow_origin: committed.attempt().workflow_origin().cloned(),
        provider_session_binding: committed.attempt().affinity().provider_session().clone(),
        idempotency: committed.attempt().idempotency(),
        commit: committed.committed().envelope().commit.clone(),
        mutation_work,
        retained_preimage: committed.retained_preimage().cloned(),
        original_authority: std::sync::Arc::clone(committed.attempt().original_authority()),
        committed_dispatch_outbox,
        output_correspondence: std::sync::Arc::new(output_correspondence),
        operation_scope: committed.attempt().affinity().operation_scope().clone(),
        observed_source_facts,
        performed_input_facts,
        performed_output_fields,
        committed_changes: crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges::from_commit(committed.committed()),
    }
}

impl WorthQueryMutationWorkCommitSeal {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPrimaryMutationWorkCounters,
        worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
        std::sync::Arc<
            worth_execution::ExecutionArray<
                crate::domain_computation::primary_graph::provider::WorthQueryTouchedRecordIdentity,
            >,
        >,
        WorthQueryPreImageRetentionWork,
    ) {
        (
            self.counters,
            self.index_maintenance_work,
            self.touched_records,
            self.preimage,
        )
    }
}

pub use postcommit_currentness::{PreparedRebaseDenial, PreparedSourceFactRebase};
