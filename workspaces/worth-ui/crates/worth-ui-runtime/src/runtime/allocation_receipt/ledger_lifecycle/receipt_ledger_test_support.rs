use crate::evidence::UiAllocationNeighborhoodScope;

pub(crate) struct UiNonPortalReceiptLawCandidate(super::UiAllocationCandidate);

impl UiNonPortalReceiptLawCandidate {
    pub(crate) fn admit(
        candidate: super::UiAllocationCandidate,
    ) -> Result<Self, Box<super::UiAllocationCandidate>> {
        if candidate.portal_allocation_input().is_some() {
            Err(Box::new(candidate))
        } else {
            Ok(Self(candidate))
        }
    }
}

pub(crate) fn detached_non_portal_receipt(
    candidate: UiNonPortalReceiptLawCandidate,
) -> super::UiAllocationReceiptCommitOutcome {
    let candidate = candidate.0;
    let verdict = match super::receipt_commit::admit_allocation_receipt_candidate(&candidate, None)
    {
        Ok(verdict) => verdict,
        Err(outcome) => return outcome,
    };
    let generation = super::UiAllocationReceiptGeneration::from_candidate(&candidate);
    let transaction =
        super::UiAllocationReplanTransaction::for_receipt_law_test(&candidate, generation);
    super::UiAllocationReceiptCommitOutcome::Committed(Box::new(
        super::receipt_commit::commit_admitted_allocation_receipt(candidate, verdict, transaction),
    ))
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiAllocationLedgerBaseline(super::ledger_state::UiAllocationReceiptLedgerState);

#[derive(Clone, Copy)]
pub(super) enum UiAllocationAuthorityExhaustionScenario {
    TransactionGeneration,
    TruthRevision { remaining: u64 },
}

impl super::UiAllocationReceiptLedger {
    pub(crate) fn position_truth_revision_for_test(
        &self,
        remaining: u64,
    ) -> UiAllocationLedgerBaseline {
        UiAllocationLedgerBaseline(self.position_exhaustion_for_test(
            UiAllocationAuthorityExhaustionScenario::TruthRevision { remaining },
        ))
    }

    pub(super) fn position_exhaustion_for_test(
        &self,
        scenario: UiAllocationAuthorityExhaustionScenario,
    ) -> super::ledger_state::UiAllocationReceiptLedgerState {
        let mut state = self.state.borrow_mut();
        match scenario {
            UiAllocationAuthorityExhaustionScenario::TransactionGeneration => {
                state.next_transaction_generation = u64::MAX;
            }
            UiAllocationAuthorityExhaustionScenario::TruthRevision { remaining } => {
                state.truth_revision = state
                    .truth_revision
                    .position_with_remaining_capacity(remaining)
                    .expect("test boundary preserves allocation truth counter invariant");
            }
        }
        assert!(state.truth_revision.invariant_holds());
        state.clone()
    }

    pub(super) fn ledger_state_for_test(
        &self,
    ) -> super::ledger_state::UiAllocationReceiptLedgerState {
        self.state.borrow().clone()
    }

    pub(crate) fn ledger_baseline_for_test(&self) -> UiAllocationLedgerBaseline {
        assert!(self.state.borrow().truth_revision.invariant_holds());
        UiAllocationLedgerBaseline(self.state.borrow().clone())
    }

    pub(crate) fn committed_scope_count(&self) -> usize {
        self.state.borrow().committed_by_scope.len()
    }

    pub(crate) fn commit_non_portal_receipt_law_candidate(
        &self,
        candidate: UiNonPortalReceiptLawCandidate,
    ) -> super::UiAllocationReceiptCommitOutcome {
        let candidate = candidate.0;
        let scope =
            UiAllocationNeighborhoodScope::from_neighborhood(candidate.allocation_neighborhood());
        let previous = self.state.borrow().committed_by_scope.get(&scope).cloned();
        let verdict = match super::receipt_commit::admit_allocation_receipt_candidate(
            &candidate,
            previous.as_ref(),
        ) {
            Ok(verdict) => verdict,
            Err(outcome) => return outcome,
        };
        let generation = super::UiAllocationReceiptGeneration::from_candidate(&candidate);
        let transaction =
            super::UiAllocationReplanTransaction::for_receipt_law_test(&candidate, generation);
        let receipt = super::receipt_commit::commit_admitted_allocation_receipt(
            candidate,
            verdict,
            transaction,
        );
        let mut state = self.state.borrow_mut();
        let predecessor_revision = state.truth_revision;
        let revision = match state.checked_truth_successor(1, false, false) {
            Ok(revision) => revision,
            Err(denial) => {
                return super::UiAllocationReceiptCommitOutcome::Denied(Box::new(
                    super::UiAllocationReceiptCommitDenial::AuthorityCounterExhausted(denial),
                ))
            }
        };
        let predecessor_projection = state.mounted_projection_catalog.clone();
        state.committed_by_scope.insert(scope, receipt.clone());
        state.mounted_projection_catalog.insert(receipt.clone());
        state.truth_revision = revision;
        let graph_node = receipt.identity().graph_node_identity();
        let changed_projection_graph_keys = state
            .mounted_projection_catalog
            .projection_changed_since(&predecessor_projection, graph_node)
            .then_some(graph_node)
            .into_iter()
            .collect();
        state.mounted_projection_journal.record(
            predecessor_revision.revision(),
            revision.revision(),
            changed_projection_graph_keys,
        );
        super::UiAllocationReceiptCommitOutcome::Committed(Box::new(receipt))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generation_bearing_constructor_is_the_complete_initial_authority() {
        let ledger = super::super::UiAllocationReceiptLedger::for_runtime_generation(73);
        let state = ledger.state.borrow();
        assert_eq!(state.runtime_generation, 73);
        assert_eq!(state.next_transaction_generation, 0);
        assert!(state.latest_frame_epoch.is_none());
        assert!(state.committed_by_scope.is_empty());
        assert!(state.completed_transactions.is_empty());
        assert!(state.denied_transactions.is_empty());
        assert!(state.durable_semantic_state.is_none());
        assert_eq!(state.truth_revision.revision(), 0);
        assert_eq!(state.truth_revision.committed_receipt_publications(), 0);
        assert_eq!(state.truth_revision.durable_resize_mutations(), 0);
        assert_eq!(state.truth_revision.durable_state_replacements(), 0);
        assert!(state.truth_revision.invariant_holds());
    }
}
