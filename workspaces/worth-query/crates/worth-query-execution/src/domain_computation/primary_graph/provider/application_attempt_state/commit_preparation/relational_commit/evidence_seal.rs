//! Exact-commit evidence minted only from the committed session stage.

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
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryMutationWorkCommitSeal {
    counters: WorthQueryPrimaryMutationWorkCounters,
    changed_records: Vec<worth_relational::facade::transactions::RecordRef>,
    preimage: WorthQueryPreImageRetentionWork,
}

pub(super) fn seal(
    committed: &WorthQueryCommittedApplicationSession,
) -> WorthQueryPrimaryGraphCommitEvidence {
    let mutation_work =
        WorthQueryPrimaryMutationWorkEvidence::from_commit_seal(WorthQueryMutationWorkCommitSeal {
            counters: committed.work(),
            changed_records: committed.committed().changed_records.clone(),
            preimage: committed.preimage_retention_work(),
        });
    let committed_dispatch_outbox = WorthQueryCommittedDispatchOutboxResolution::from_commit(
        committed.attempt().dispatch_outbox(),
        committed.committed(),
    );
    WorthQueryPrimaryGraphCommitEvidence {
        provider_session_binding: committed.attempt().affinity().provider_session().clone(),
        idempotency: committed.attempt().idempotency(),
        commit: committed.committed().envelope().commit.clone(),
        mutation_work,
        retained_preimage: committed.retained_preimage().cloned(),
        committed_dispatch_outbox,
    }
}

impl WorthQueryMutationWorkCommitSeal {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        WorthQueryPrimaryMutationWorkCounters,
        Vec<worth_relational::facade::transactions::RecordRef>,
        WorthQueryPreImageRetentionWork,
    ) {
        (self.counters, self.changed_records, self.preimage)
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
}
