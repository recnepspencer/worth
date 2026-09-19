use super::transaction::{prepare_active_successor, PreparedActiveSuccessor};
use super::UiCommittedAllocationSuccessors;

pub(crate) enum UiCommittedAllocationPreflightDenial {
    ActivationGate {
        denial: Box<crate::runtime::WorthUiActivationGateDenial>,
        counters: Box<super::UiCommittedAllocationActivationCounters>,
    },
    CandidatePlanDigestMismatch {
        counters: Box<super::UiCommittedAllocationActivationCounters>,
    },
    LedgerCommittedOutcomeMismatch {
        counters: Box<super::UiCommittedAllocationActivationCounters>,
    },
    CounterExhausted {
        counters: Box<super::UiCommittedAllocationActivationCounters>,
        exhaustion: super::UiCommittedAllocationActivationCounterExhaustion,
    },
}

pub(crate) struct UiPreflightedCommittedAllocationTransaction {
    attempt_identity_digest: u64,
    committed_row_count: usize,
    prior_observation: crate::runtime::WorthUiPriorValidPlanObservation,
    gate_receipt: crate::runtime::WorthUiActivationGateReceipt,
    counters: super::UiCommittedAllocationActivationCounters,
    previous: crate::runtime::WorthUiActiveRuntimeObservation,
    next_active: crate::runtime::active::WorthUiActiveRuntimeState,
    ledger_transition: crate::runtime::allocation_receipt::UiAllocationCatalogLedgerTransition,
    committed_allocation: crate::runtime::UiCommittedAllocationReplan,
    structural_reuse: crate::runtime::WorthUiPlanRegionalEvidence,
}

pub(crate) struct UiCommitTruthResources {
    transaction: UiPreflightedCommittedAllocationTransaction,
    ledger_commit: crate::runtime::allocation_receipt::UiPreparedAllocationCatalogLedgerCommit,
}

pub(super) fn preflight_committed_allocation(
    active: &crate::runtime::active::WorthUiActiveRuntimeState,
    ready: super::UiCommittedAllocationValidation,
    candidate_bundle: crate::runtime::active::WorthUiSealedExecutionPlanBundle,
    boundary: crate::runtime::WorthUiFrameBoundary,
    runtime_frame_epoch: crate::runtime::WorthUiRuntimeFrameEpoch,
    runtime_host_session: crate::facade::WorthUiHostSessionIdentity,
) -> Result<UiPreflightedCommittedAllocationTransaction, UiCommittedAllocationPreflightDenial> {
    let mut counters = ready.activation_counters();
    let attempt_identity_digest = ready.attempt_identity().structural_digest();
    let committed_row_count = ready.attempt_identity().committed_row_count();
    let prior = super::WorthUiPriorValidPlan::capture(active);
    let prior_observation = prior.observation();
    let gate_receipt = super::frame_validation::validate_frame_boundary(
        active.observation(),
        &ready,
        boundary,
        runtime_frame_epoch,
        runtime_host_session,
    )
    .map_err(
        |denial| UiCommittedAllocationPreflightDenial::ActivationGate {
            denial: Box::new(denial),
            counters: Box::new(counters),
        },
    )?;
    counters
        .record_active_successor_build()
        .map_err(
            |exhaustion| UiCommittedAllocationPreflightDenial::CounterExhausted {
                counters: Box::new(counters),
                exhaustion,
            },
        )?;
    let structural_reuse = candidate_bundle
        .execution_plan()
        .regional_evidence()
        .clone();
    let payload =
        PreparedActiveSuccessor::prepare(active, ready, candidate_bundle).map_err(|_| {
            UiCommittedAllocationPreflightDenial::CandidatePlanDigestMismatch {
                counters: Box::new(counters),
            }
        })?;
    let previous = active.observation();
    let (next_active, ledger_transition, committed) =
        prepare_active_successor(active, payload, runtime_frame_epoch);
    if &committed != ledger_transition.committed_outcome() {
        return Err(
            UiCommittedAllocationPreflightDenial::LedgerCommittedOutcomeMismatch {
                counters: Box::new(counters),
            },
        );
    }
    counters.record_live_mutation().map_err(|exhaustion| {
        UiCommittedAllocationPreflightDenial::CounterExhausted {
            counters: Box::new(counters),
            exhaustion,
        }
    })?;
    Ok(UiPreflightedCommittedAllocationTransaction {
        attempt_identity_digest,
        committed_row_count,
        prior_observation,
        gate_receipt,
        counters,
        previous,
        next_active,
        ledger_transition,
        committed_allocation: committed,
        structural_reuse,
    })
}

impl UiPreflightedCommittedAllocationTransaction {
    pub(crate) fn activation_counters(&self) -> super::UiCommittedAllocationActivationCounters {
        self.counters
    }

    pub(crate) fn acquire_truth_resources(
        self,
        ledger: &crate::runtime::allocation_receipt::UiAllocationReceiptLedger,
    ) -> Result<UiCommitTruthResources, Box<Self>> {
        let Some(ledger_commit) = ledger.prepare_catalog_commit(&self.ledger_transition) else {
            return Err(Box::new(self));
        };
        Ok(UiCommitTruthResources {
            transaction: self,
            ledger_commit,
        })
    }

    pub(crate) fn seal(
        self,
        scroll_catalog_evidence: crate::runtime::UiScrollCatalogSwapEvidence,
        invalidation_transition: crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    ) -> UiCommittedAllocationSuccessors {
        let next = self.next_active.observation();
        let receipt_draft = super::WorthUiPlanSwapReceiptDraft {
            attempt_identity_digest: self.attempt_identity_digest,
            committed_row_count: self.committed_row_count,
            previous_active_artifact_digest: self.previous.artifact_digest(),
            previous_active_plan_digest: self.previous.active_plan_digest(),
            previous_active_snapshot_digest: self.previous.snapshot_digest(),
            next_active_artifact_digest: next.artifact_digest(),
            next_active_plan_digest: next.active_plan_digest(),
            next_active_snapshot_digest: next.snapshot_digest(),
            activation_gate_receipt: self.gate_receipt,
            prior_valid_plan: self.prior_observation,
            counters: self.counters,
            scroll_catalog_evidence,
            committed_allocation: self.committed_allocation,
            structural_reuse: self.structural_reuse,
        };
        UiCommittedAllocationSuccessors::new(
            receipt_draft,
            self.next_active,
            self.ledger_transition,
            invalidation_transition,
        )
    }
}

impl UiCommitTruthResources {
    pub(crate) fn seal(
        self,
        scroll_catalog_evidence: crate::runtime::UiScrollCatalogSwapEvidence,
        invalidation_transition: crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    ) -> (
        UiCommittedAllocationSuccessors,
        crate::runtime::allocation_receipt::UiPreparedAllocationCatalogLedgerCommit,
    ) {
        let prepared = self
            .transaction
            .seal(scroll_catalog_evidence, invalidation_transition);
        (prepared, self.ledger_commit)
    }
}
