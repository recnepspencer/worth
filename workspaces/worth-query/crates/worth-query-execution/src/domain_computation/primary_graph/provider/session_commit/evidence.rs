//! Immutable exact-commit evidence retained for equivalent receipt resolution.

use std::collections::{BTreeMap, BTreeSet};
use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::super::WorthQueryPrimaryGraphCommittedApplication;
use crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold;

/// Immutable owner evidence indexed by the Relational commit that produced it.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCompletedCommitEvidenceStore {
    by_commit: BTreeMap<CommitId, WorthQueryPrimaryGraphCommittedApplication>,
    by_dispatch_correlation: BTreeMap<
        crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
        BTreeSet<CommitId>,
    >,
    by_session:
        BTreeMap<crate::domain_computation::WorthQueryProviderSessionAffinityIdentity, CommitId>,
    by_idempotency:
        BTreeMap<(super::super::WorthQueryProductIdempotencyAffinity, [u8; 32]), CommitId>,
    /// Evidence still holding its performed publication, by World commit.
    performed_by_world: BTreeMap<CompositeCommitIdentity, CommitId>,
    /// The World history hold each recorded commit's publication handed over,
    /// kept until that commit's own caller resolves its receipt and takes it.
    fresh_history: BTreeMap<CommitId, WorthQueryCommitHistoryHold>,
}

/// The selectors one evidence entry is indexed under besides its commit.
struct EvidenceKeys {
    affinity: crate::domain_computation::WorthQueryProviderSessionAffinityIdentity,
    idempotency: (super::super::WorthQueryProductIdempotencyAffinity, [u8; 32]),
    dispatch_correlation:
        Option<crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity>,
}

impl EvidenceKeys {
    fn of(evidence: &WorthQueryPrimaryGraphCommittedApplication) -> Self {
        Self {
            affinity: evidence
                .commit_evidence()
                .provider_session_binding()
                .affinity_identity(),
            idempotency: (
                super::super::WorthQueryProductIdempotencyAffinity::from_publication(
                    evidence.committed_product_publication(),
                ),
                *evidence.commit_evidence().idempotency().key_identity(),
            ),
            dispatch_correlation: evidence
                .commit_evidence()
                .committed_dispatch_outbox()
                .seal_for_receipt()
                .ok()
                .and_then(|seal| seal.into_binding())
                .map(|binding| *binding.record().correlation()),
        }
    }
}

impl WorthQueryCompletedCommitEvidenceStore {
    pub(in crate::domain_computation::primary_graph::provider) fn take_one_for_product_occurrence(
        &mut self,
        branch: &worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Option<(CommitId, WorthQueryPrimaryGraphCommittedApplication)> {
        let commit = self.by_commit.iter().find_map(|(commit, evidence)| {
            let publication = evidence.committed_product_publication();
            (publication.product_branch() == branch
                && publication.product_incarnation() == incarnation)
                .then_some(*commit)
        })?;
        let evidence = self.remove(commit)?;
        Some((commit, evidence))
    }

    fn remove(&mut self, commit: CommitId) -> Option<WorthQueryPrimaryGraphCommittedApplication> {
        let evidence = self.by_commit.remove(&commit)?;
        let keys = EvidenceKeys::of(&evidence);
        if self.by_session.get(&keys.affinity) == Some(&commit) {
            self.by_session.remove(&keys.affinity);
        }
        if self.by_idempotency.get(&keys.idempotency) == Some(&commit) {
            self.by_idempotency.remove(&keys.idempotency);
        }
        if let Some(correlation) = keys.dispatch_correlation {
            if let Some(indexed) = self.by_dispatch_correlation.get_mut(&correlation) {
                indexed.remove(&commit);
                if indexed.is_empty() {
                    self.by_dispatch_correlation.remove(&correlation);
                }
            }
        }
        let world = evidence.committed_product_publication().composite_commit();
        if self.performed_by_world.get(world) == Some(&commit) {
            self.performed_by_world.remove(world);
        }
        self.fresh_history.remove(&commit);
        Some(evidence)
    }

    /// Hands `evidence`'s commit's World history hold to its own caller.
    pub(in crate::domain_computation::primary_graph::provider) fn claim_fresh_history(
        &mut self,
        evidence: &mut WorthQueryPrimaryGraphCommittedApplication,
    ) {
        if let Some(hold) = self
            .fresh_history
            .remove(&evidence.commit_reference().commit_id)
        {
            evidence.hold_history(hold);
        }
    }

    #[cfg(feature = "test-query-execution-observer")]
    pub(in crate::domain_computation::primary_graph) fn entry_count(&self) -> usize {
        self.by_commit.len()
    }

    /// Evidence outlives its commit's history: once that history retires it
    /// keeps only the publication's identities, so replay still answers and
    /// nothing in World or its owners stays pinned. Returns the Relational
    /// commits whose evidence retired.
    pub(in crate::domain_computation::primary_graph::provider) fn retire(
        &mut self,
        retired: &[CompositeCommitIdentity],
    ) -> Vec<CommitId> {
        retired
            .iter()
            .filter_map(|world| {
                let commit = self.performed_by_world.remove(world)?;
                self.by_commit.get_mut(&commit)?.retire_publication();
                Some(commit)
            })
            .collect()
    }

    pub(in crate::domain_computation::primary_graph::provider) fn record(
        &mut self,
        evidence: WorthQueryPrimaryGraphCommittedApplication,
    ) {
        // Evidence outlives its commit's history, so it never protects it;
        // the hold waits for the commit's own caller to take it.
        let mut evidence = evidence;
        let hold = evidence.take_history();
        let commit = evidence.commit_reference().commit_id;
        let EvidenceKeys {
            affinity,
            idempotency: idempotency_key,
            dispatch_correlation,
        } = EvidenceKeys::of(&evidence);
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
        self.performed_by_world.insert(
            evidence
                .committed_product_publication()
                .composite_commit()
                .clone(),
            commit,
        );
        self.by_session.insert(affinity, commit);
        self.by_idempotency.insert(idempotency_key, commit);
        if let Some(correlation) = dispatch_correlation {
            self.by_dispatch_correlation
                .entry(correlation)
                .or_default()
                .insert(commit);
        }
        if let Some(hold) = hold {
            self.fresh_history.insert(commit, hold);
        }
        self.by_commit.insert(commit, evidence);
    }

    /// The correlation is only a selector. The caller must still read the
    /// exact Relational row and check its performed World publication.
    pub(in crate::domain_computation::primary_graph::provider) fn observe_correlation(
        &self,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Result<Option<WorthQueryPrimaryGraphCommittedApplication>, ()> {
        let Some(commits) = self.by_dispatch_correlation.get(correlation) else {
            return Ok(None);
        };
        if commits.len() != 1 {
            return Err(());
        }
        let commit = commits.iter().next().expect("one indexed commit");
        self.by_commit.get(commit).cloned().map(Some).ok_or(())
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
