use super::{
    ledger_denial::{denied, retain_denial},
    ledger_state::UiAllocationReceiptLedgerState,
};
use super::{
    UiAllocationReplanTransactionCommitDenial, UiAllocationReplanTransactionCounters,
    UiAllocationReplanTransactionOutcome, UiAllocationReuseVerdict, UiCommittedAllocationReplan,
};
use {crate::evidence::UiAllocationNeighborhoodScope, std::cell::RefCell};
#[derive(Debug)]
pub(crate) struct UiAllocationReceiptLedger {
    pub(super) state: RefCell<UiAllocationReceiptLedgerState>,
}
impl UiAllocationReceiptLedger {
    pub(crate) fn for_runtime_generation(runtime_generation: u64) -> Self {
        Self {
            state: RefCell::new(UiAllocationReceiptLedgerState::initial(runtime_generation)),
        }
    }

    pub(crate) fn mounted_projection_source(
        &self,
        predecessor_revision: Option<u64>,
    ) -> super::UiMountedAllocationProjectionSource {
        let state = self.state.borrow();
        state.mounted_projection_journal.source(
            state.mounted_projection_catalog.clone(),
            predecessor_revision,
            state.truth_revision.revision(),
        )
    }

    pub(crate) fn active_catalog_is_empty(&self) -> bool {
        self.state.borrow().committed_by_scope.is_empty()
    }

    pub(super) fn prepare_selected_mode(
        &self,
        mode: super::replan_commit_mode::UiAllocationReplanCommitMode<'_>,
    ) -> super::UiAllocationLedgerPreparation {
        let selection = mode.selection();
        if let Err(denial) =
            super::receipt_budget::admit_replan_budget(selection, mode.durable_resize().is_some())
        {
            return denied(denial).into();
        }
        let candidate_result = match mode.durable_resize() {
            Some(basis) => {
                crate::runtime::planning::replan_selected_candidates_with_resize(selection, basis)
            }
            None => crate::runtime::planning::replan_selected_candidates_with_portal(selection),
        };
        let candidates = match candidate_result {
            Ok(candidates) => candidates,
            Err(ordinal) => {
                return denied(
                    UiAllocationReplanTransactionCommitDenial::CandidatePlanningDenied { ordinal },
                )
                .into()
            }
        };
        let mut state = self.state.borrow_mut();
        let generation = state.checked_transaction_generation();
        let transaction = match super::UiAllocationReplanTransaction::from_graph_basis(
            selection.transaction_basis(),
            generation.unwrap_or(state.next_transaction_generation),
            state.runtime_generation,
        ) {
            Ok(transaction) => transaction,
            Err(_) => {
                return denied(UiAllocationReplanTransactionCommitDenial::TransactionIdentityDenied)
                    .into()
            }
        };
        let replay_key = transaction.idempotency_key();
        if let Some(previous) = state
            .completed_transactions
            .get(&replay_key)
            .and_then(|bucket| {
                bucket
                    .iter()
                    .find(|completed| completed.transaction().same_idempotency_basis(&transaction))
            })
        {
            return UiAllocationReplanTransactionOutcome::Replayed(previous.clone()).into();
        }
        if let Some(previous) = state
            .denied_transactions
            .get(&replay_key)
            .and_then(|bucket| {
                bucket.iter().find_map(|(denied_transaction, denial)| {
                    denied_transaction
                        .same_idempotency_basis(&transaction)
                        .then_some(denial)
                })
            })
        {
            return UiAllocationReplanTransactionOutcome::Denied(*previous).into();
        }
        let generation = match generation {
            Ok(generation) => generation,
            Err(denial) => {
                return denied(
                    UiAllocationReplanTransactionCommitDenial::AuthorityCounterExhausted(denial),
                )
                .into()
            }
        };
        if state
            .latest_frame_epoch
            .is_some_and(|latest| transaction.frame_epoch() < latest)
        {
            return retain_denial(
                &mut state,
                &transaction,
                UiAllocationReplanTransactionCommitDenial::StaleTransactionFrame,
            );
        }
        let mut verdicts = Vec::with_capacity(candidates.len());
        let Ok(mut counters) = UiAllocationReplanTransactionCounters::preflight(candidates.len())
        else {
            return retain_denial(
                &mut state,
                &transaction,
                UiAllocationReplanTransactionCommitDenial::EvidenceCounterExhausted,
            );
        };
        for (ordinal, (selected, candidate)) in selection
            .ordered_neighborhoods()
            .iter()
            .zip(candidates.iter())
            .enumerate()
        {
            if selected.identity() != candidate.allocation_neighborhood().identity() {
                return retain_denial(
                    &mut state,
                    &transaction,
                    UiAllocationReplanTransactionCommitDenial::CandidateNeighborhoodMismatch {
                        ordinal: ordinal as u16,
                    },
                );
            }
            if !candidate.is_admitted() {
                return retain_denial(
                    &mut state,
                    &transaction,
                    UiAllocationReplanTransactionCommitDenial::CandidatePlanningDenied {
                        ordinal: ordinal as u16,
                    },
                );
            }
            let scope = UiAllocationNeighborhoodScope::from_neighborhood(
                candidate.allocation_neighborhood(),
            );
            let previous = state.committed_by_scope.get(&scope);
            if let Some(portal) = candidate.portal_allocation_input() {
                let prior_matches = previous.is_some_and(|receipt| {
                    receipt.identity() == portal.prior_receipt_identity()
                        && receipt.generation() == portal.prior_receipt_generation()
                });
                if !prior_matches {
                    return retain_denial(
                        &mut state,
                        &transaction,
                        UiAllocationReplanTransactionCommitDenial::PortalPriorReceiptMismatch {
                            ordinal: ordinal as u16,
                        },
                    );
                }
            }
            let mut verdict = previous.map_or(UiAllocationReuseVerdict::NewCommit, |receipt| {
                super::receipt_commit::evaluate_allocation_receipt_reuse(candidate, receipt)
            });
            if mode.durable_resize().is_some()
                && matches!(
                    verdict,
                    UiAllocationReuseVerdict::Denied(
                        super::UiAllocationReuseDenial::EquivalenceBasisMismatch
                    )
                )
            {
                verdict = UiAllocationReuseVerdict::NewCommit;
            }
            if mode.admits_query_measurement_successor(selected.identity())
                && matches!(
                    verdict,
                    UiAllocationReuseVerdict::Denied(
                        super::UiAllocationReuseDenial::EquivalenceBasisMismatch
                    )
                )
            {
                verdict = UiAllocationReuseVerdict::NewCommit;
            }
            if mode.admits_host_measurement_successor(selected.identity())
                && matches!(
                    verdict,
                    UiAllocationReuseVerdict::Denied(
                        super::UiAllocationReuseDenial::GenerationMismatch
                            | super::UiAllocationReuseDenial::EquivalenceBasisMismatch
                            | super::UiAllocationReuseDenial::UnsupportedPartialReuse
                    )
                )
            {
                verdict = UiAllocationReuseVerdict::NewCommit;
            }
            if candidate.portal_allocation_input().is_some()
                && matches!(verdict, UiAllocationReuseVerdict::Denied(_))
            {
                verdict = UiAllocationReuseVerdict::NewCommit;
            }
            let counted = match verdict {
                UiAllocationReuseVerdict::Denied(reason) => {
                    return retain_denial(
                        &mut state,
                        &transaction,
                        UiAllocationReplanTransactionCommitDenial::ReuseDenied {
                            ordinal: ordinal as u16,
                            reason,
                        },
                    )
                }
                UiAllocationReuseVerdict::StructureReuseLeafRemeasure(_) => {
                    return retain_denial(
                        &mut state,
                        &transaction,
                        UiAllocationReplanTransactionCommitDenial::RecomputePending {
                            ordinal: ordinal as u16,
                        },
                    )
                }
                UiAllocationReuseVerdict::FullReuse => counters.reused(),
                UiAllocationReuseVerdict::NewCommit => counters.replanned(),
            };
            if counted.is_err() {
                return retain_denial(
                    &mut state,
                    &transaction,
                    UiAllocationReplanTransactionCommitDenial::EvidenceCounterExhausted,
                );
            }
            verdicts.push(verdict);
        }
        drop(state);
        let (committed, successor_candidates) =
            match Self::commit_candidates(&mode, transaction, candidates, verdicts, counters) {
                Ok(committed) => committed,
                Err(denial) => return UiAllocationReplanTransactionOutcome::Denied(denial).into(),
            };
        self.prepare_transition(
            &mode,
            generation,
            replay_key,
            committed,
            successor_candidates,
        )
    }

    fn commit_candidates(
        mode: &super::replan_commit_mode::UiAllocationReplanCommitMode<'_>,
        transaction: super::UiAllocationReplanTransaction,
        candidates: Vec<crate::runtime::UiAllocationCandidate>,
        verdicts: Vec<UiAllocationReuseVerdict>,
        mut counters: UiAllocationReplanTransactionCounters,
    ) -> Result<
        (
            UiCommittedAllocationReplan,
            Vec<crate::runtime::UiAllocationCandidate>,
        ),
        UiAllocationReplanTransactionCommitDenial,
    > {
        let catalog_candidates = candidates.clone();
        let receipts = candidates
            .into_iter()
            .zip(verdicts)
            .map(|(candidate, verdict)| {
                super::receipt_commit::commit_admitted_allocation_receipt(
                    candidate,
                    verdict,
                    transaction.clone(),
                )
            })
            .collect::<Vec<_>>();
        counters
            .committed(receipts.len())
            .map_err(|_| UiAllocationReplanTransactionCommitDenial::EvidenceCounterExhausted)?;
        let catalog_bindings =
            super::UiCommittedAllocationCatalogBindings::seal(&catalog_candidates, &receipts)
                .map_err(|_| UiAllocationReplanTransactionCommitDenial::CatalogBindingMismatch)?;
        let committed =
            UiCommittedAllocationReplan::new(transaction, receipts, counters, catalog_bindings)
                .map_err(|_| UiAllocationReplanTransactionCommitDenial::EvidenceCounterExhausted)?;
        let committed = match mode {
            super::replan_commit_mode::UiAllocationReplanCommitMode::Viewport(basis) => {
                super::viewport_inspection::attach_viewport_inspection(committed, basis)
            }
            _ => committed,
        };
        Ok((committed, catalog_candidates))
    }

    fn prepare_transition(
        &self,
        mode: &super::replan_commit_mode::UiAllocationReplanCommitMode<'_>,
        generation: u64,
        replay_key: u64,
        committed: UiCommittedAllocationReplan,
        successor_candidates: Vec<crate::runtime::UiAllocationCandidate>,
    ) -> super::UiAllocationLedgerPreparation {
        let committed_frame_epoch = committed.transaction().frame_epoch();
        let predecessor = self.state.borrow().clone();
        let mut successor = predecessor.clone();
        let resize_mutated = mode.durable_resize().is_some_and(|basis| {
            successor
                .durable_semantic_state
                .as_ref()
                .and_then(|state| state.committed_resize(basis.durable_identity_digest()))
                != Some(basis)
        });
        successor.truth_revision = match successor.checked_truth_successor(
            committed.receipts().len(),
            resize_mutated,
            false,
        ) {
            Ok(revision) => revision,
            Err(denial) => {
                return denied(
                    UiAllocationReplanTransactionCommitDenial::AuthorityCounterExhausted(denial),
                )
                .into()
            }
        };
        if successor
            .latest_frame_epoch
            .is_none_or(|latest| committed_frame_epoch > latest)
        {
            successor.latest_frame_epoch = Some(committed_frame_epoch);
        }
        for receipt in committed.receipts() {
            let scope = UiAllocationNeighborhoodScope::from_neighborhood(
                receipt.committed_allocation().allocation_neighborhood(),
            );
            successor.committed_by_scope.insert(scope, receipt.clone());
            successor.mounted_projection_catalog.insert(receipt.clone());
        }
        let changed_projection_graph_keys = committed
            .receipts()
            .iter()
            .map(|receipt| receipt.identity().graph_node_identity())
            .filter(|graph_node| {
                successor
                    .mounted_projection_catalog
                    .projection_changed_since(&predecessor.mounted_projection_catalog, *graph_node)
            })
            .collect();
        successor.mounted_projection_journal.record(
            predecessor.truth_revision.revision(),
            successor.truth_revision.revision(),
            changed_projection_graph_keys,
        );
        successor.next_transaction_generation = generation;
        if let Some(basis) = mode.durable_resize() {
            let mutated = successor
                .durable_semantic_state
                .as_mut()
                .expect("durable commit requires activated reconciliation state")
                .commit_resize(basis.clone());
            debug_assert_eq!(mutated, resize_mutated);
        }
        successor
            .completed_transactions
            .entry(replay_key)
            .or_default()
            .push(committed.clone());
        super::UiAllocationLedgerPreparation::Prepared(Box::new(
            super::UiPreparedAllocationLedgerTransition::new(
                predecessor,
                successor,
                committed,
                successor_candidates,
            ),
        ))
    }
}
