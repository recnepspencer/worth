use super::{
    ledger_state::UiAllocationCatalogLedgerTransition, UiAllocationReceiptLedger,
    UiAllocationReplanTransactionCounters, UiAllocationReuseVerdict, UiCommittedAllocationReplan,
};
use crate::evidence::UiAllocationNeighborhoodScope;

impl UiAllocationReceiptLedger {
    pub(crate) fn seal_activation_catalog(
        &self,
        catalog: crate::runtime::invalidation_narrowing::UiAllocationActivationCatalog,
        frame_epoch: crate::runtime::UiAllocationFrameEpoch,
        reconciliation: &crate::runtime::WorthUiDurableStateReconciliationPlan,
    ) -> Result<
        crate::runtime::UiCommittedAllocationActivationAttempt,
        super::UiAllocationReceiptCommitOutcome,
    > {
        let candidates = catalog.candidates_for_commit();
        for (ordinal, candidate) in candidates.iter().enumerate() {
            let ordinal = u16::try_from(ordinal).map_err(|_| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::CatalogBindingCardinalityMismatch,
                )
            })?;
            let admission = candidate.replan_admission_opt().ok_or({
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::catalog_activation(
                        super::UiCommittedAllocationCatalogActivationDenial::MissingReplanAdmission {
                            ordinal,
                        },
                    ),
                )
            })?;
            admission.committed_scroll_sources().map_err(|denial| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::catalog_activation(
                        super::UiCommittedAllocationCatalogActivationDenial::ScrollAuthority {
                            ordinal,
                            denial,
                        },
                    ),
                )
            })?;
        }
        let state = self.state.borrow();
        let transaction_generation = state.checked_transaction_generation();
        let transaction = super::UiAllocationReplanTransaction::for_replacement_activation(
            candidates,
            state.runtime_generation,
            transaction_generation.unwrap_or(state.next_transaction_generation),
            frame_epoch,
        )
        .map_err(|_| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::candidate_planning(
                    super::UiAllocationReceiptDenialReport::candidate_planning_denied(
                        candidates
                            .first()
                            .expect("catalog admission proves non-empty"),
                    ),
                ),
            )
        })?;
        let replay_key = transaction.idempotency_key();
        if let Some(previous) = state
            .completed_transactions
            .get(&replay_key)
            .and_then(|bucket| {
                bucket
                    .iter()
                    .find(|completed| completed.transaction().same_idempotency_basis(&transaction))
            })
            .cloned()
        {
            let predecessor = state.clone();
            let mut successor = predecessor.clone();
            let durable = reconciliation.allocation_durable_semantic_state();
            let replacement = successor.durable_semantic_state.as_ref() != Some(&durable);
            successor.truth_revision = successor
                .checked_truth_successor(0, false, replacement)
                .map_err(|denial| {
                    super::UiAllocationReceiptCommitOutcome::denied(
                        super::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(denial),
                    )
                })?;
            if replacement {
                successor.durable_semantic_state = Some(durable);
            }
            let bindings =
                super::UiCommittedAllocationCatalogBindings::seal(candidates, previous.receipts())
                    .map_err(super::UiAllocationReceiptCommitOutcome::denied)?;
            let activation =
                super::UiCommittedAllocationCatalogActivation::seal(candidates, &bindings)
                    .map_err(|denial| {
                        super::UiAllocationReceiptCommitOutcome::denied(
                            super::UiAllocationReceiptCommitDenial::catalog_activation(denial),
                        )
                    })?;
            let committed = previous.clone();
            let allocation_changed = !active_catalog_semantically_matches(&successor, &committed);
            retain_only_active_catalog(&mut successor, &committed, replay_key);
            return Ok(crate::runtime::UiCommittedAllocationActivationAttempt::new(
                catalog,
                UiAllocationCatalogLedgerTransition {
                    predecessor,
                    successor,
                    outcome: previous,
                    durable_reconciliation: reconciliation.clone(),
                    operational_meaning_changed: replacement || allocation_changed,
                },
                committed,
                activation,
            ));
        }
        transaction_generation.map_err(|denial| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(denial),
            )
        })?;
        let mut verdicts = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let scope = UiAllocationNeighborhoodScope::from_neighborhood(
                candidate.allocation_neighborhood(),
            );
            verdicts.push(
                super::receipt_commit::admit_replacement_allocation_receipt_candidate(
                    candidate,
                    state.committed_by_scope.get(&scope),
                )?,
            );
        }
        drop(state);
        let mut counters = UiAllocationReplanTransactionCounters::preflight(candidates.len())
            .map_err(|()| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted,
                )
            })?;
        for verdict in &verdicts {
            let counted = match verdict {
                UiAllocationReuseVerdict::FullReuse => counters.reused(),
                UiAllocationReuseVerdict::NewCommit => counters.replanned(),
                UiAllocationReuseVerdict::StructureReuseLeafRemeasure(_)
                | UiAllocationReuseVerdict::Denied(_) => {
                    unreachable!("catalog preflight admitted only publishable verdicts")
                }
            };
            counted.map_err(|()| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted,
                )
            })?;
        }
        counters.committed(candidates.len()).map_err(|()| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted,
            )
        })?;
        let counter_report = super::UiAllocationCounterReport::from_commit(&transaction, counters);
        let receipts = candidates
            .iter()
            .cloned()
            .zip(verdicts)
            .map(|(candidate, verdict)| {
                let mut receipt = super::receipt_commit::commit_admitted_allocation_receipt(
                    candidate,
                    verdict,
                    transaction.clone(),
                );
                receipt.attach_counter_report(counter_report.clone());
                receipt
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let predecessor = self.state.borrow().clone();
        let mut successor = predecessor.clone();
        let durable = reconciliation.allocation_durable_semantic_state();
        let replacement = successor.durable_semantic_state.as_ref() != Some(&durable);
        successor.truth_revision = successor
            .checked_truth_successor(receipts.len(), false, replacement)
            .map_err(|denial| {
                super::UiAllocationReceiptCommitOutcome::denied(
                    super::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(denial),
                )
            })?;
        if replacement {
            successor.durable_semantic_state = Some(durable);
        }
        let receipts = receipts.into_vec();
        let catalog_bindings =
            super::UiCommittedAllocationCatalogBindings::seal(candidates, &receipts)
                .map_err(super::UiAllocationReceiptCommitOutcome::denied)?;
        let outcome = UiCommittedAllocationReplan::new(
            transaction.clone(),
            receipts,
            counters,
            catalog_bindings,
        )
        .map_err(|()| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::EvidenceCounterExhausted,
            )
        })?;
        let has_runtime_consequences = !outcome
            .transaction()
            .consequences()
            .scroll_owned()
            .is_empty()
            || !outcome
                .transaction()
                .consequences()
                .portal_anchors()
                .is_empty();
        let allocation_changed = !active_catalog_semantically_matches(&successor, &outcome);
        let activation = super::UiCommittedAllocationCatalogActivation::seal(
            candidates,
            outcome.catalog_bindings(),
        )
        .map_err(|denial| {
            super::UiAllocationReceiptCommitOutcome::denied(
                super::UiAllocationReceiptCommitDenial::catalog_activation(denial),
            )
        })?;
        successor.next_transaction_generation = transaction.transaction_generation();
        successor.latest_frame_epoch = Some(frame_epoch);
        retain_only_active_catalog(&mut successor, &outcome, replay_key);
        let committed = outcome.clone();
        Ok(crate::runtime::UiCommittedAllocationActivationAttempt::new(
            catalog,
            UiAllocationCatalogLedgerTransition {
                predecessor,
                successor,
                outcome,
                durable_reconciliation: reconciliation.clone(),
                operational_meaning_changed: replacement
                    || allocation_changed
                    || has_runtime_consequences,
            },
            committed,
            activation,
        ))
    }
}

fn active_catalog_semantically_matches(
    predecessor: &super::ledger_state::UiAllocationReceiptLedgerState,
    committed: &UiCommittedAllocationReplan,
) -> bool {
    predecessor.committed_by_scope.len() == committed.receipts().len()
        && committed.receipts().iter().all(|candidate| {
            let scope = UiAllocationNeighborhoodScope::from_neighborhood(
                candidate.committed_allocation().allocation_neighborhood(),
            );
            predecessor
                .committed_by_scope
                .get(&scope)
                .is_some_and(|active| active.operationally_matches(candidate))
        })
}

fn retain_only_active_catalog(
    successor: &mut super::ledger_state::UiAllocationReceiptLedgerState,
    committed: &UiCommittedAllocationReplan,
    replay_key: u64,
) {
    successor.committed_by_scope = Default::default();
    successor
        .mounted_projection_catalog
        .replace_with(committed.receipts());
    for receipt in committed.receipts() {
        successor.committed_by_scope.insert(
            UiAllocationNeighborhoodScope::from_neighborhood(
                receipt.committed_allocation().allocation_neighborhood(),
            ),
            receipt.clone(),
        );
    }
    successor.completed_transactions.clear();
    successor
        .completed_transactions
        .insert(replay_key, vec![committed.clone()]);
    successor.denied_transactions.clear();
}
