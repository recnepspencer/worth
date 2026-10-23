//! Linear continuation from one committed undo into possible redo admission.

use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;

use super::recovery_handle::{RelinquishOnDenial, WorthQueryHeldRecoveryHandle};
use super::{
    WorthQueryAftermathDerivationFailure, WorthQueryProvedUndo, WorthQueryRecoveryHandle,
    WorthQueryUndoProgressionHandoff,
};

/// Redo holds the handle inside the sealed continuation, so every redo denial —
/// retirement, copied intent, changed meaning — would otherwise drop it and destroy
/// the commit's recovery instead of leaving it available for a corrected
/// attempt (Q8.21-L11).
///
/// As with undo admission, dropping is the whole implementation: the
/// continuation carries its handle in a [`WorthQueryHeldRecoveryHandle`], which
/// also covers the host's own denials in `admit_redo_disbursement_recovery`
/// (Q8.22-C5).
impl RelinquishOnDenial for WorthQueryRedoRecovery {
    fn relinquish_held_handle(self) {
        drop(self);
    }
}

/// Descriptive proved-undo evidence paired with its framework-owned handle.
///
/// Possession grants no current authority. The private pairing prevents a
/// caller from combining one proved undo with an unrelated recovery handle.
#[derive(Debug)]
pub struct WorthQueryRedoRecovery {
    proved: WorthQueryProvedUndo,
    handle: WorthQueryHeldRecoveryHandle,
}

impl WorthQueryRedoRecovery {
    /// Seal the redo continuation only after ordinary undo commit succeeds.
    pub fn from_completed_undo(
        handoff: WorthQueryUndoProgressionHandoff,
        undo_receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Self, WorthQueryAftermathDerivationFailure> {
        let sealed = WorthQueryProvedUndo::seal_completed(handoff.admission(), undo_receipt);
        // Take the handle out before answering, so the handoff can no longer
        // relinquish it. Reaching here means the ordinary undo *committed*; the
        // commit's one recovery is therefore exercised whether or not redo
        // evidence could be sealed on top of it. Relinquishing would offer a
        // second undo of a commit that has already been undone.
        let handle = handoff.into_recovery_handle();
        match sealed {
            Ok(proved) => Ok(Self {
                proved,
                handle: WorthQueryHeldRecoveryHandle::new(handle),
            }),
            Err(failure) => {
                let _ = handle.consume(WorthQueryRecoveryResourceTerminal::Consumed);
                Err(failure)
            }
        }
    }

    pub const fn proved(&self) -> &WorthQueryProvedUndo {
        &self.proved
    }

    pub fn handle(&self) -> &WorthQueryRecoveryHandle {
        self.handle.get()
    }

    pub(crate) fn into_parts(self) -> (WorthQueryProvedUndo, WorthQueryRecoveryHandle) {
        (self.proved, self.handle.into_handle())
    }

    #[cfg(test)]
    pub(crate) fn axis_probe(
        proved: WorthQueryProvedUndo,
        handle: WorthQueryRecoveryHandle,
    ) -> Self {
        Self {
            proved,
            handle: WorthQueryHeldRecoveryHandle::new(handle),
        }
    }
}
