use crate::evidence::UiAllocationNeighborhoodScope;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiAllocationReceiptLedgerState {
    pub(super) committed_by_scope: crate::runtime::persistent_index::UiPersistentOrdMap<
        UiAllocationNeighborhoodScope,
        super::UiAllocationReceipt,
    >,
    pub(super) mounted_projection_catalog: super::UiMountedAllocationProjectionCatalog,
    pub(super) mounted_projection_journal:
        super::mounted_projection_journal::UiMountedAllocationProjectionJournal,
    pub(super) completed_transactions: BTreeMap<u64, Vec<super::UiCommittedAllocationReplan>>,
    pub(super) denied_transactions: BTreeMap<
        u64,
        Vec<(
            super::UiAllocationReplanTransaction,
            super::UiAllocationReplanTransactionCommitDenial,
        )>,
    >,
    pub(super) latest_frame_epoch: Option<crate::runtime::UiAllocationFrameEpoch>,
    pub(super) next_transaction_generation: u64,
    pub(super) runtime_generation: u64,
    pub(super) durable_semantic_state: Option<super::UiAllocationDurableSemanticState>,
    pub(super) truth_revision: super::UiAllocationTruthRevision,
}

impl UiAllocationReceiptLedgerState {
    pub(super) fn initial(runtime_generation: u64) -> Self {
        Self {
            committed_by_scope: Default::default(),
            mounted_projection_catalog: Default::default(),
            mounted_projection_journal: Default::default(),
            completed_transactions: BTreeMap::new(),
            denied_transactions: BTreeMap::new(),
            latest_frame_epoch: None,
            next_transaction_generation: 0,
            runtime_generation,
            durable_semantic_state: None,
            truth_revision: super::UiAllocationTruthRevision::initial(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct UiAllocationCatalogLedgerTransition {
    pub(super) predecessor: UiAllocationReceiptLedgerState,
    pub(super) successor: UiAllocationReceiptLedgerState,
    pub(super) outcome: super::UiCommittedAllocationReplan,
    pub(super) durable_reconciliation: crate::runtime::WorthUiDurableStateReconciliationPlan,
    pub(super) operational_meaning_changed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAllocationCatalogLedgerLineage {
    predecessor_truth: super::UiAllocationTruthRevision,
    successor_truth: super::UiAllocationTruthRevision,
    predecessor_transaction_generation: u64,
    successor_transaction_generation: u64,
    runtime_generation: u64,
}

impl UiAllocationCatalogLedgerLineage {
    pub(crate) fn identity_digest(&self) -> u64 {
        self.predecessor_truth.revision()
            ^ self.successor_truth.revision().rotate_left(7)
            ^ self.predecessor_transaction_generation.rotate_left(17)
            ^ self.successor_transaction_generation.rotate_left(29)
            ^ self.runtime_generation.rotate_left(43)
    }
}

impl UiAllocationCatalogLedgerTransition {
    pub(crate) fn predecessor_receipt(
        &self,
        scope: &crate::evidence::UiAllocationNeighborhoodScope,
    ) -> Option<&super::UiAllocationReceipt> {
        self.predecessor.committed_by_scope.get(scope)
    }

    pub(crate) fn committed_outcome(&self) -> &super::UiCommittedAllocationReplan {
        &self.outcome
    }
    pub(crate) fn structural_lineage(&self) -> UiAllocationCatalogLedgerLineage {
        UiAllocationCatalogLedgerLineage {
            predecessor_truth: self.predecessor.truth_revision,
            successor_truth: self.successor.truth_revision,
            predecessor_transaction_generation: self.predecessor.next_transaction_generation,
            successor_transaction_generation: self.successor.next_transaction_generation,
            runtime_generation: self.predecessor.runtime_generation,
        }
    }
    pub(crate) fn durable_reconciliation(
        &self,
    ) -> &crate::runtime::WorthUiDurableStateReconciliationPlan {
        &self.durable_reconciliation
    }

    pub(crate) fn operational_meaning_unchanged(&self) -> bool {
        !self.operational_meaning_changed
    }

    pub(crate) fn successor_allocation_identity_digest(&self, projection_digest: u64) -> u64 {
        self.successor.truth_revision.revision().rotate_left(17)
            ^ self.successor.committed_by_scope.len().rotate_left(31) as u64
            ^ projection_digest.rotate_left(43)
    }

    pub(crate) fn apply_successor_delta(
        &mut self,
        affected: &[crate::evidence::UiAllocationNeighborhoodScope],
        changed: &[super::UiAllocationReceipt],
    ) {
        let mut complete = self.predecessor.committed_by_scope.clone();
        let mut projection = self.predecessor.mounted_projection_catalog.clone();
        let mut candidate_graph_nodes = Vec::new();
        for scope in affected {
            if let Some(receipt) = complete.get(scope) {
                candidate_graph_nodes.push(receipt.identity().graph_node_identity());
                projection.remove(receipt);
            }
            complete.remove(scope);
        }
        for receipt in changed {
            candidate_graph_nodes.push(receipt.identity().graph_node_identity());
            complete.insert(
                crate::evidence::UiAllocationNeighborhoodScope::from_neighborhood(
                    receipt.committed_allocation().allocation_neighborhood(),
                ),
                receipt.clone(),
            );
            projection.insert(receipt.clone());
        }
        candidate_graph_nodes.sort();
        candidate_graph_nodes.dedup();
        let changed_projection_graph_keys = candidate_graph_nodes
            .into_iter()
            .filter(|graph_node| {
                projection.projection_changed_since(
                    &self.predecessor.mounted_projection_catalog,
                    *graph_node,
                )
            })
            .collect();
        self.successor.committed_by_scope = complete;
        self.successor.mounted_projection_catalog = projection;
        self.successor.mounted_projection_journal.record(
            self.predecessor.truth_revision.revision(),
            self.successor.truth_revision.revision(),
            changed_projection_graph_keys,
        );
    }
}
