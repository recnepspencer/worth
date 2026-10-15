use crate::runtime::allocation_receipt::{
    UiAllocationLedgerPreparation, UiAllocationReceiptLedger, UiPreparedAllocationLedgerTransition,
};

#[derive(Debug)]
pub(in crate::runtime) struct UiAllocationTransactionAuthority {
    graph_basis: crate::graph::UiGraphReplanTransactionBasis,
    transaction: Option<crate::runtime::UiAllocationReplanTransaction>,
}

#[derive(Debug)]
pub(super) struct UiPendingAllocationTransaction {
    authority: UiAllocationTransactionAuthority,
    preparation: UiAllocationLedgerPreparation,
}

pub(super) fn prepare_selected(
    ledger: &UiAllocationReceiptLedger,
    authority: &crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
) -> UiPendingAllocationTransaction {
    let transaction_authority = UiAllocationTransactionAuthority::for_selection(selection);
    let preparation = if authority.certifies_selection(selection) {
        ledger.prepare_selected(&transaction_authority, selection)
    } else { match ledger
        .completed_replay(selection, None)
        .filter(|replay| authority.certifies_completed_replay(replay))
    { Some(replay) => {
        replay.into()
    } _ => {
        denied_generation().into()
    }}};
    UiPendingAllocationTransaction {
        authority: transaction_authority,
        preparation,
    }
}

pub(super) fn prepare_viewport(
    ledger: &UiAllocationReceiptLedger,
    authority: &crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    basis: crate::runtime::UiViewportResizeCommitBasis,
) -> UiPendingAllocationTransaction {
    let transaction_authority = UiAllocationTransactionAuthority::for_selection(basis.selection());
    let preparation = if authority.certifies_selection(basis.selection()) {
        ledger.prepare_viewport(&transaction_authority, basis)
    } else { match ledger
        .completed_replay(basis.selection(), None)
        .filter(|replay| authority.certifies_completed_replay(replay))
    { Some(replay) => {
        replay.into()
    } _ => {
        denied_generation().into()
    }}};
    UiPendingAllocationTransaction {
        authority: transaction_authority,
        preparation,
    }
}

impl UiAllocationTransactionAuthority {
    fn for_selection(selection: &crate::graph::UiAdmittedReplanNeighborhoodSet) -> Self {
        Self {
            graph_basis: selection.transaction_basis().clone(),
            transaction: None,
        }
    }

    pub(in crate::runtime) fn certifies_selection(
        &self,
        selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
    ) -> bool {
        self.graph_basis == *selection.transaction_basis()
    }

    fn bind_transition(&mut self, transition: &UiPreparedAllocationLedgerTransition) {
        self.transaction = Some(transition.committed().transaction().clone());
    }

    pub(in crate::runtime) fn certifies_committed(
        &self,
        committed: &crate::runtime::UiCommittedAllocationReplan,
    ) -> bool {
        self.transaction.as_ref().is_some_and(|expected| {
            expected.same_idempotency_basis(committed.transaction())
                && expected == committed.transaction()
        })
    }
}

pub(super) fn prepare_pending_durable_resize(
    ledger: &UiAllocationReceiptLedger,
    authority: &crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    selection: &crate::graph::UiAdmittedReplanNeighborhoodSet,
    identity: u64,
    extent: crate::runtime::UiResizeLogicalExtent,
) -> (UiPendingAllocationTransaction, bool) {
    let transaction_authority = UiAllocationTransactionAuthority::for_selection(selection);
    let (preparation, requested_mutation) = if authority.certifies_selection(selection) {
        let (preparation, _, requested_mutation) =
            ledger.prepare_durable_resize(&transaction_authority, selection, identity, extent);
        (preparation, requested_mutation)
    } else if let Some(resize) = crate::runtime::UiResizeAllocationPlanningBasis::seal(
        selection,
        selection.primary().locality().target_graph_node_identity(),
        Some(identity),
        extent,
    ) {
        ledger
            .completed_replay(selection, Some(&resize))
            .filter(|replay| authority.certifies_completed_replay(replay))
            .map_or_else(
                || (denied_generation().into(), false),
                |replay| (replay.into(), false),
            )
    } else {
        (denied_generation().into(), false)
    };
    (
        UiPendingAllocationTransaction {
            authority: transaction_authority,
            preparation,
        },
        requested_mutation,
    )
}

pub(super) fn publish_pending(
    ledger: &UiAllocationReceiptLedger,
    authority: &mut crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    mut pending: UiPendingAllocationTransaction,
) -> crate::runtime::UiAllocationReplanTransactionOutcome {
    publish_prepared(
        ledger,
        authority,
        &mut pending.authority,
        pending.preparation,
    )
}

pub(super) fn publish_prepared(
    ledger: &UiAllocationReceiptLedger,
    authority: &mut crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    transaction_authority: &mut UiAllocationTransactionAuthority,
    preparation: UiAllocationLedgerPreparation,
) -> crate::runtime::UiAllocationReplanTransactionOutcome {
    let UiAllocationLedgerPreparation::Prepared(transition) = preparation else {
        let UiAllocationLedgerPreparation::Resolved(outcome) = preparation else {
            unreachable!()
        };
        return *outcome;
    };
    let transition = *transition;
    transaction_authority.bind_transition(&transition);
    let (transition, portal_succession) =
        match attach_portal_succession(ledger, authority, transaction_authority, transition) {
            Ok(prepared) => prepared,
            Err(outcome) => return *outcome,
        };
    let authority_succession = match prepare_authority_succession(authority, &transition) {
        Ok(prepared) => prepared,
        Err(denial) => {
            return ledger.retain_prepared_denial(
                &transition,
                crate::runtime::UiAllocationReplanTransactionCommitDenial::AllocationAuthoritySuccession(
                    denial,
                ),
            )
        }
    };
    let transition = match attach_query_scroll_succession(ledger, transition, &authority_succession)
    {
        Ok(transition) => transition,
        Err(outcome) => return *outcome,
    };
    bind_and_publish(
        ledger,
        authority,
        transition,
        portal_succession,
        authority_succession,
        transaction_authority,
    )
}

fn attach_portal_succession(
    ledger: &UiAllocationReceiptLedger,
    authority: &crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    transaction_authority: &UiAllocationTransactionAuthority,
    transition: UiPreparedAllocationLedgerTransition,
) -> Result<
    (
        UiPreparedAllocationLedgerTransition,
        Option<crate::runtime::invalidation_narrowing::UiPreparedPortalBindingSuccession>,
    ),
    Box<crate::runtime::UiAllocationReplanTransactionOutcome>,
> {
    let portal = !transition
        .committed()
        .transaction()
        .consequences()
        .portal_anchors()
        .is_empty();
    if !portal {
        return Ok((transition, None));
    }
    match authority.prepare_portal_binding_succession(transaction_authority, transition.committed())
    {
        Ok(prepared) => {
            let committed = transition
                .committed()
                .clone()
                .with_portal_binding_succession(prepared.receipt());
            Ok((transition.with_committed(committed), Some(prepared)))
        }
        Err(denial) => Err(Box::new(ledger.retain_prepared_denial(
            &transition,
            crate::runtime::UiAllocationReplanTransactionCommitDenial::PortalBindingSuccession(
                denial,
            ),
        ))),
    }
}

fn attach_query_scroll_succession(
    ledger: &UiAllocationReceiptLedger,
    transition: UiPreparedAllocationLedgerTransition,
    authority_succession: &crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
) -> Result<
    UiPreparedAllocationLedgerTransition,
    Box<crate::runtime::UiAllocationReplanTransactionOutcome>,
> {
    let query_measurement = !transition
        .committed()
        .transaction()
        .consequences()
        .query_measurements()
        .is_empty();
    if !query_measurement {
        return Ok(transition);
    }
    let receipt = match authority_succession.scroll_catalog_evidence() {
        crate::runtime::UiScrollCatalogSwapEvidence::Prepared(receipt) => receipt,
        crate::runtime::UiScrollCatalogSwapEvidence::Denied(_) => {
            return Err(Box::new(ledger.retain_prepared_denial(
                &transition,
                crate::runtime::UiAllocationReplanTransactionCommitDenial::AllocationAuthoritySuccession(
                    crate::runtime::UiAllocationAuthoritySuccessionDenial::ScrollBinding,
                ),
            )))
        }
    };
    let committed = transition
        .committed()
        .clone()
        .with_scroll_binding_succession(receipt);
    Ok(transition.with_committed(committed))
}

fn prepare_authority_succession(
    authority: &crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    transition: &UiPreparedAllocationLedgerTransition,
) -> Result<
    crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    crate::runtime::UiAllocationAuthoritySuccessionDenial,
> {
    let activation =
        crate::runtime::allocation_receipt::UiCommittedAllocationCatalogActivation::seal(
            transition.successor_candidates(),
            transition.committed().catalog_bindings(),
        )
        .map_err(map_catalog_activation_denial)?;
    let mut affected = transition
        .successor_candidates()
        .iter()
        .map(|candidate| {
            crate::evidence::UiAllocationNeighborhoodScope::from_neighborhood(
                candidate.allocation_neighborhood(),
            )
        })
        .collect::<Vec<_>>();
    affected.sort();
    affected.dedup();
    let delta = authority.seal_catalog_transition(activation, Some(affected.into_boxed_slice()));
    authority
        .prepare_catalog_transition(&delta)
        .map_err(map_catalog_transition_denial)
}

fn bind_and_publish(
    ledger: &UiAllocationReceiptLedger,
    authority: &mut crate::runtime::invalidation_narrowing::UiAllocationInvalidationAuthority,
    transition: UiPreparedAllocationLedgerTransition,
    portal_succession: Option<
        crate::runtime::invalidation_narrowing::UiPreparedPortalBindingSuccession,
    >,
    authority_succession:
        crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    transaction_authority: &mut UiAllocationTransactionAuthority,
) -> crate::runtime::UiAllocationReplanTransactionOutcome {
    if let Some(prepared) = portal_succession.as_ref() {
        let expected = prepared.predecessor_identity_digest();
        let observed = authority.portal_binding_identity_digest();
        if expected != observed {
            return crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
                crate::runtime::UiAllocationReplanTransactionCommitDenial::PortalCommitBind(
                    crate::runtime::UiPortalAllocationCommitBindDenial::BindingPredecessorChanged {
                        expected_identity_digest: expected,
                        observed_identity_digest: observed,
                    },
                ),
            );
        }
    }
    if authority_succession.predecessor_identity_digest()
        != authority.active_catalog_identity_digest()
    {
        return crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
            crate::runtime::UiAllocationReplanTransactionCommitDenial::AllocationAuthoritySuccession(
                crate::runtime::UiAllocationAuthoritySuccessionDenial::StalePredecessor,
            ),
        );
    }
    let outcome = transition.committed().clone();
    let ledger_commit = match ledger.bind_replan_transition(transaction_authority, &transition) {
        Ok(prepared) => prepared,
        Err(denial) => {
            return crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
                crate::runtime::UiAllocationReplanTransactionCommitDenial::PortalCommitBind(denial),
            )
        }
    };
    let transaction_authority = ledger_commit.commit_once();
    let _ = transaction_authority;
    let _ = portal_succession;
    authority.commit_catalog_transition(authority_succession);
    crate::runtime::UiAllocationReplanTransactionOutcome::Committed(outcome)
}

fn map_catalog_activation_denial(
    denial: crate::runtime::allocation_receipt::UiCommittedAllocationCatalogActivationDenial,
) -> crate::runtime::UiAllocationAuthoritySuccessionDenial {
    use crate::runtime::allocation_receipt::UiCommittedAllocationCatalogActivationDenial as Activation;
    use crate::runtime::UiAllocationAuthoritySuccessionDenial as Succession;
    match denial {
        Activation::CardinalityMismatch => Succession::CatalogCardinalityMismatch,
        Activation::MissingReplanAdmission { ordinal } => {
            Succession::MissingReplanAdmission { ordinal }
        }
        Activation::ScrollAuthority { ordinal, .. } => Succession::ScrollAuthority { ordinal },
        Activation::PortalAuthority { ordinal } => Succession::PortalAuthority { ordinal },
    }
}

fn map_catalog_transition_denial(
    denial: crate::runtime::invalidation_narrowing::UiAllocationNeighborhoodActivationDenial,
) -> crate::runtime::UiAllocationAuthoritySuccessionDenial {
    use crate::runtime::invalidation_narrowing::UiAllocationNeighborhoodActivationDenial as Denial;
    use crate::runtime::UiAllocationAuthoritySuccessionDenial as Succession;
    match denial {
        Denial::StalePredecessor => Succession::StalePredecessor,
        Denial::DerivedIndexDiverged => Succession::DerivedIndexDiverged,
        Denial::ScrollBinding(_) => Succession::ScrollBinding,
        Denial::PortalBinding(_) => Succession::PortalBinding,
    }
}

fn denied_generation() -> crate::runtime::UiAllocationReplanTransactionOutcome {
    crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
        crate::runtime::UiAllocationReplanTransactionCommitDenial::AdmittedGenerationSetChanged,
    )
}
