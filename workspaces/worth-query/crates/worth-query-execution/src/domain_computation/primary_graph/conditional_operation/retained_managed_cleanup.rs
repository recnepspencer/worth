//! Independent native cleanup never makes a conditional operation runnable again.
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCleanupRequired,
    WorthQueryApplicationManagedCleanup, WorthQueryApplicationUncommitted,
};
pub(super) struct RetainedManagedCleanup {
    outcome: WorthQueryApplicationUncommitted,
    pending: Option<WorthQueryApplicationManagedCleanup>,
    receipt: Option<crate::domain_computation::WorthQueryDirectRunCleanupReceipt>,
    kind: crate::domain_computation::WorthQueryMutationCleanupFailureKind,
}
impl RetainedManagedCleanup {
    pub(super) fn new(required: Box<WorthQueryApplicationCleanupRequired>) -> Self {
        let (outcome, cleanup) = required.into_parts();
        let kind = cleanup.failure().kind();
        Self {
            outcome,
            pending: Some(cleanup),
            receipt: None,
            kind,
        }
    }
    pub(super) fn run_identity(&self) -> &str {
        match &self.pending {
            Some(cleanup) => cleanup.failure().run_identity(),
            None => self
                .receipt
                .as_ref()
                .expect("released cleanup retains its actual receipt")
                .inspection()
                .run_identity(),
        }
    }
    pub(super) fn kind(&self) -> crate::domain_computation::WorthQueryMutationCleanupFailureKind {
        self.kind
    }
    pub(super) fn pending_kind(
        &self,
    ) -> Option<crate::domain_computation::WorthQueryMutationCleanupFailureKind> {
        self.pending
            .as_ref()
            .map(|cleanup| cleanup.failure().kind())
    }
    pub(super) fn retry(&mut self, phase: &WorthQueryAdvancementPhase<'_>) {
        if let Some(cleanup) = self.pending.take() {
            match cleanup.retry_in_advancement(phase) {
                Ok(receipt) => self.receipt = Some(receipt),
                Err(cleanup) => self.pending = Some(cleanup),
            }
        }
        // Retain both native business evidence and release disposition. Neither
        // a physical release nor reinstallation settles the business attempt.
        let _ = (&self.outcome, &self.receipt);
    }
}

use super::signal_decision_reentry::{
    WorthQueryRetainedConditionalDecision as Decision, WorthQueryRetainedConditionalWake as Wake,
};
pub(super) fn pending_kind<'a>(
    wakes: impl Iterator<Item = &'a Wake>,
) -> Option<crate::domain_computation::WorthQueryMutationCleanupFailureKind> {
    wakes
        .filter_map(|wake| match &wake.decision {
            Decision::OperationCleanupRequired(_, cleanup) => cleanup.pending_kind(),
            _ => None,
        })
        .next()
}
pub(super) fn retain_cleanup_wakes(wakes: &mut Vec<Wake>) {
    wakes.retain(|wake| matches!(wake.decision, Decision::OperationCleanupRequired(_, _)));
}
pub(super) fn has_cleanup_wakes(wakes: &[Wake]) -> bool {
    wakes
        .iter()
        .any(|wake| matches!(wake.decision, Decision::OperationCleanupRequired(_, _)))
}

pub(super) fn admit_reinstallation(
    kind: Option<crate::domain_computation::WorthQueryMutationCleanupFailureKind>,
    identity: &str,
) -> Result<(), super::installation::WorthQueryConditionalRuntimeInstallationDenial> {
    if let Some(kind) = kind {
        return Err(super::installation::WorthQueryConditionalRuntimeInstallationDenial::new(
            super::installation::WorthQueryConditionalRuntimeInstallationDenialKind::ManagedCleanupRetained(kind), identity));
    }
    Ok(())
}

pub(super) fn retry_wakes<'a>(
    wakes: impl Iterator<Item = &'a mut Wake>,
    phase: &WorthQueryAdvancementPhase<'_>,
) {
    for wake in wakes {
        if let Decision::OperationCleanupRequired(_, cleanup) = &mut wake.decision {
            cleanup.retry(phase);
        }
    }
}
pub(super) fn include_retained_cleanup<'a>(
    receipt: &mut super::clock_observation::ErasedClockObservationReceipt,
    wakes: impl Iterator<Item = &'a Wake>,
) {
    for wake in
        wakes.filter(|wake| matches!(wake.decision, Decision::OperationCleanupRequired(_, _)))
    {
        receipt.retained_due_wake_count += 1;
        receipt.retained_failed_wake_count += 1;
        receipt
            .execution_provenance
            .extend(super::execution_provenance::execution_provenance(
                std::slice::from_ref(wake),
            ));
    }
}
