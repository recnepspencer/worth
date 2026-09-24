//! Exact-commit evidence minted only from the committed session stage.

mod postcommit_currentness;

use super::commit_execution::WorthQueryCommittedApplicationSession;
use crate::domain_computation::primary_graph::provider::{
    mutation_work::{WorthQueryPrimaryMutationWorkCounters, WorthQueryPrimaryMutationWorkEvidence},
    session_commit::{
        WorthQueryCommittedDispatchOutboxResolution, WorthQueryPreImageRetentionWork,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryPrimaryGraphCommitEvidence {
    provider_session_binding:
        crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    idempotency:
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    commit: worth_relational::facade::history::RelationalCommitReceipt,
    mutation_work: WorthQueryPrimaryMutationWorkEvidence,
    retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    committed_dispatch_outbox: WorthQueryCommittedDispatchOutboxResolution,
    committed_changes: crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges,
    output_correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
    operation_scope: crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    observed_source_facts: std::sync::Arc<
        [crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact],
    >,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryMutationWorkCommitSeal {
    counters: WorthQueryPrimaryMutationWorkCounters,
    index_maintenance_work: worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
    changed_records: Vec<worth_relational::facade::transactions::RecordRef>,
    preimage: WorthQueryPreImageRetentionWork,
}

pub(super) fn seal(
    provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    committed: &WorthQueryCommittedApplicationSession,
) -> WorthQueryPrimaryGraphCommitEvidence {
    let mutation_work =
        WorthQueryPrimaryMutationWorkEvidence::from_commit_seal(WorthQueryMutationWorkCommitSeal {
            counters: committed.work(),
            index_maintenance_work: committed.index_maintenance_work(),
            changed_records: committed.committed().changed_records.clone(),
            preimage: committed.preimage_retention_work(),
        });
    let committed_dispatch_outbox = WorthQueryCommittedDispatchOutboxResolution::from_commit(
        committed.attempt().dispatch_outbox(),
        committed.committed(),
    );
    let output_correspondence = committed
        .attempt()
        .seal_output_correspondence(committed.committed());
    let observed_source_facts = provider.graph.with_runtime(|runtime| {
        postcommit_currentness::rebase(
            runtime,
            &committed.committed().snapshot,
            committed.attempt().observed_source_facts(),
        )
    });
    WorthQueryPrimaryGraphCommitEvidence {
        provider_session_binding: committed.attempt().affinity().provider_session().clone(),
        idempotency: committed.attempt().idempotency(),
        commit: committed.committed().envelope().commit.clone(),
        mutation_work,
        retained_preimage: committed.retained_preimage().cloned(),
        committed_dispatch_outbox,
        output_correspondence: std::sync::Arc::new(output_correspondence),
        operation_scope: committed.attempt().affinity().operation_scope().clone(),
        observed_source_facts,
        committed_changes: crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges::from_commit(committed.committed()),
    }
}

impl WorthQueryMutationWorkCommitSeal {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        WorthQueryPrimaryMutationWorkCounters,
        worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
        Vec<worth_relational::facade::transactions::RecordRef>,
        WorthQueryPreImageRetentionWork,
    ) {
        (
            self.counters,
            self.index_maintenance_work,
            self.changed_records,
            self.preimage,
        )
    }
}

impl WorthQueryPrimaryGraphCommitEvidence {
    pub(in crate::domain_computation::primary_graph) const fn provider_session_binding(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding
    {
        &self.provider_session_binding
    }

    pub(in crate::domain_computation::primary_graph) const fn idempotency(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding
    {
        self.idempotency
    }

    pub(in crate::domain_computation::primary_graph) const fn commit_reference(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitReceipt {
        &self.commit
    }

    pub(in crate::domain_computation::primary_graph) const fn mutation_work(
        &self,
    ) -> &WorthQueryPrimaryMutationWorkEvidence {
        &self.mutation_work
    }

    pub(in crate::domain_computation::primary_graph) const fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) const fn committed_dispatch_outbox(
        &self,
    ) -> &WorthQueryCommittedDispatchOutboxResolution {
        &self.committed_dispatch_outbox
    }

    pub(in crate::domain_computation::primary_graph) const fn committed_changes(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges {
        &self.committed_changes
    }

    pub(in crate::domain_computation::primary_graph) fn output_correspondence(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        self.output_correspondence.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_output_correspondence(
        &self,
    ) -> std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    > {
        std::sync::Arc::clone(&self.output_correspondence)
    }

    pub(in crate::domain_computation::primary_graph) const fn operation_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        &self.operation_scope
    }

    pub(in crate::domain_computation::primary_graph) fn retain_observed_source_facts(
        &self,
    ) -> std::sync::Arc<
        [crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact],
    >{
        std::sync::Arc::clone(&self.observed_source_facts)
    }
}
