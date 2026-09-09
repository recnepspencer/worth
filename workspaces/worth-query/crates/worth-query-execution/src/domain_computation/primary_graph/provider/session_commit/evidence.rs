//! Immutable exact-commit evidence retained for equivalent receipt resolution.

use std::collections::BTreeMap;
use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};

use super::super::WorthQueryPrimaryGraphCommittedApplication;

/// Immutable owner evidence indexed by the Relational commit that produced it.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCompletedCommitEvidenceStore {
    by_commit: BTreeMap<CommitId, WorthQueryPrimaryGraphCommittedApplication>,
    by_session:
        BTreeMap<crate::domain_computation::WorthQueryProviderSessionAffinityIdentity, CommitId>,
    by_idempotency:
        BTreeMap<(super::super::WorthQueryProductIdempotencyAffinity, [u8; 32]), CommitId>,
}

impl WorthQueryCompletedCommitEvidenceStore {
    pub(in crate::domain_computation::primary_graph::provider) fn record(
        &mut self,
        evidence: WorthQueryPrimaryGraphCommittedApplication,
    ) {
        let commit = evidence.commit_reference().commit_id;
        let affinity = evidence
            .commit_evidence()
            .provider_session_binding()
            .affinity_identity();
        let idempotency = evidence.commit_evidence().idempotency();
        let idempotency_key = (
            super::super::WorthQueryProductIdempotencyAffinity::from_reference(
                evidence.product_publication().new_product_head(),
            ),
            *idempotency.key_identity(),
        );
        assert!(
            !self.by_session.contains_key(&affinity),
            "one provider session may record completed evidence only once"
        );
        assert!(
            !self.by_commit.contains_key(&commit),
            "one Relational commit may record provider evidence only once"
        );
        assert!(
            !self.by_idempotency.contains_key(&idempotency_key),
            "one product-occurrence-qualified idempotency key may record completed evidence only once"
        );
        self.by_session.insert(affinity, commit);
        self.by_idempotency.insert(idempotency_key, commit);
        self.by_commit.insert(commit, evidence);
    }

    pub(in crate::domain_computation::primary_graph) fn observe(
        &self,
        commit: &RelationalCommitReceipt,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        self.by_commit
            .get(&commit.commit_id)
            .filter(|evidence| evidence.commit_reference() == commit)
            .cloned()
    }

    pub(in crate::domain_computation::primary_graph) fn observe_session(
        &self,
        session: &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        let commit = self.by_session.get(&session.affinity_identity())?;
        self.by_commit
            .get(commit)
            .filter(|evidence| {
                evidence
                    .commit_evidence()
                    .provider_session_binding()
                    .same_session(session)
            })
            .cloned()
    }

    pub(in crate::domain_computation::primary_graph) fn observe_idempotency(
        &self,
        product: &super::super::WorthQueryProductIdempotencyAffinity,
        binding: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    ) -> Option<(
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
        WorthQueryPrimaryGraphCommittedApplication,
    )>{
        let commit = self
            .by_idempotency
            .get(&(product.clone(), *binding.key_identity()))?;
        let evidence = self.by_commit.get(commit)?;
        Some((evidence.commit_evidence().idempotency(), evidence.clone()))
    }
}
