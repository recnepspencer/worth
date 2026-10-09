//! Ordinary progression handoff for an admitted redo (R8.43).
//!
//! Redo is an ordinary operation with an unusual input. Admission derives
//! nothing about current authority from the intent; this module only hands the
//! admitted request into ordinary mutation progression.

use super::redo_admission::WorthQueryRedoAdmission;
use super::redo_denial::WorthQueryRedoDenial;
use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryRetainedGovernedInput,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::redo_intent::WorthQueryRedoIntent;
use super::redo_recovery::WorthQueryRedoRecovery;

/// Sealed handoff from redo admission into ordinary mutation progression.
#[derive(Debug)]
pub struct WorthQueryRedoProgressionHandoff {
    recovery: WorthQueryRedoRecovery,
    intent: WorthQueryRedoIntent,
    retained_governed_input: WorthQueryRetainedGovernedInput,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    redo_admission_work: WorthQueryCanonicalWorkEvidence,
    _private: (),
}

impl WorthQueryRedoProgressionHandoff {
    pub(crate) fn recovery_handle(&self) -> &super::WorthQueryRecoveryHandle {
        self.recovery.handle()
    }
    pub const fn intent(&self) -> &WorthQueryRedoIntent {
        &self.intent
    }

    pub const fn redo_admission_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.redo_admission_work
    }

    pub fn original_input<Input: 'static>(&self) -> Option<&Input> {
        self.retained_governed_input.downcast_ref()
    }

    pub const fn idempotency_binding(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    pub(crate) fn pending_causality(&self) -> super::WorthQueryPendingAftermathCausality {
        super::WorthQueryPendingAftermathCausality::redo_of(self.intent.undo_commit().clone())
    }

    /// Return the exact proved-undo continuation after a zero-effect outcome.
    pub fn into_retry_recovery(self) -> WorthQueryRedoRecovery {
        self.recovery
    }
}

/// Progress an admitted redo into the ordinary mutation lane.
pub fn progress_admitted_redo(
    admission: WorthQueryRedoAdmission,
) -> Result<WorthQueryRedoProgressionHandoff, WorthQueryRedoDenial> {
    let (recovery, intent, retained_governed_input, idempotency, redo_admission_work) =
        admission.into_progression_parts();
    Ok(WorthQueryRedoProgressionHandoff {
        recovery,
        intent,
        retained_governed_input,
        idempotency,
        redo_admission_work,
        _private: (),
    })
}

/// Close a redo continuation only after an effect may have committed.
pub fn consume_redo_progression(
    handoff: WorthQueryRedoProgressionHandoff,
) -> Result<(), WorthQueryRedoDenial> {
    let (_, handle) = handoff.recovery.into_parts();
    handle
        .consume(WorthQueryRecoveryResourceTerminal::Consumed)
        .map(|_| ())
        .map_err(|denial| super::redo_admission::map_recovery_denial(denial.kind()))
}

/// Map an ordinary compare-and-commit denial into a redo denial cause.
pub fn map_ordinary_commit_conflict_to_redo() -> WorthQueryRedoDenial {
    WorthQueryRedoDenial::conflicted()
}
