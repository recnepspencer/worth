use super::{
    ledger_state::UiAllocationCatalogLedgerTransition, UiAllocationReceiptLedger,
    UiAllocationReplanTransactionCounters, UiCommittedAllocationReplan,
};

impl UiAllocationReceiptLedger {
    pub(crate) fn seal_removal_only_catalog_activation(
        &self,
        catalog: crate::runtime::invalidation_narrowing::UiAllocationActivationCatalog,
        frame_epoch: crate::runtime::UiAllocationFrameEpoch,
        reconciliation: &crate::runtime::WorthUiDurableStateReconciliationPlan,
        affected: &[crate::evidence::UiAllocationNeighborhoodScope],
    ) -> Result<
        crate::runtime::UiCommittedAllocationActivationAttempt,
        super::UiAllocationReceiptCommitOutcome,
    > {
        let state = self.state.borrow();
        let removed_identities = affected
            .iter()
            .map(|scope| {
                state.committed_by_scope.get(scope).map(|receipt| {
                    receipt
                        .committed_allocation()
                        .allocation_neighborhood()
                        .identity()
                        .clone()
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(binding_denial)?
            .into_boxed_slice();
        let generation = state.checked_transaction_generation().map_err(|denial| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(denial),
            )
        })?;
        let transaction = super::UiAllocationReplanTransaction::for_catalog_removal_activation(
            removed_identities,
            removal_overlap_disposition(affected),
            state.runtime_generation,
            generation,
            frame_epoch,
        )
        .map_err(|_| binding_denial())?;
        let counters =
            UiAllocationReplanTransactionCounters::preflight(affected.len()).map_err(|()| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted,
                )
            })?;
        let bindings = super::UiCommittedAllocationCatalogBindings::seal(&[], &[])
            .map_err(super::UiAllocationReceiptCommitOutcome::denied)?;
        let outcome =
            UiCommittedAllocationReplan::new(transaction.clone(), Vec::new(), counters, bindings)
                .map_err(|()| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted,
                )
            })?;
        let activation = super::UiCommittedAllocationCatalogActivation::seal(
            catalog.candidates_for_commit(),
            outcome.catalog_bindings(),
        )
        .map_err(|denial| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::catalog_activation(denial),
            )
        })?;
        let predecessor = state.clone();
        let mut successor = predecessor.clone();
        let durable = reconciliation.allocation_durable_semantic_state();
        let durable_changed = successor.durable_semantic_state.as_ref() != Some(&durable);
        successor.truth_revision =
            successor
                .checked_truth_successor(0, false, true)
                .map_err(|denial| {
                    super::UiAllocationReceiptCommitOutcome::denied(
                        super::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(denial),
                    )
                })?;
        if durable_changed {
            successor.durable_semantic_state = Some(durable);
        }
        successor.next_transaction_generation = transaction.transaction_generation();
        successor.latest_frame_epoch = Some(frame_epoch);
        successor.completed_transactions = Default::default();
        successor
            .completed_transactions
            .insert(transaction.idempotency_key(), vec![outcome.clone()]);
        successor.denied_transactions = Default::default();
        drop(state);
        Ok(crate::runtime::UiCommittedAllocationActivationAttempt::new(
            catalog,
            UiAllocationCatalogLedgerTransition {
                predecessor,
                successor,
                outcome: outcome.clone(),
                durable_reconciliation: reconciliation.clone(),
                operational_meaning_changed: true,
            },
            outcome,
            activation,
        ))
    }
}

fn removal_overlap_disposition(
    affected: &[crate::evidence::UiAllocationNeighborhoodScope],
) -> crate::graph::UiReplanOverlapDisposition {
    if affected.len() == 1 {
        return crate::graph::UiReplanOverlapDisposition::Singleton;
    }
    let pairwise_disjoint = affected.iter().enumerate().all(|(ordinal, left)| {
        affected[ordinal + 1..].iter().all(|right| {
            !left.member_identity_digests().iter().any(|member| {
                right
                    .member_identity_digests()
                    .binary_search(member)
                    .is_ok()
            })
        })
    });
    if pairwise_disjoint {
        crate::graph::UiReplanOverlapDisposition::PairwiseDisjoint
    } else {
        crate::graph::UiReplanOverlapDisposition::ContainmentMerged
    }
}

fn binding_denial() -> super::UiAllocationReceiptCommitOutcome {
    super::UiAllocationReceiptCommitOutcome::denied(
        super::UiAllocationReceiptCommitDenial::CatalogBindingCardinalityMismatch,
    )
}
