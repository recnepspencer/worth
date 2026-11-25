//! Outcome carrier and custody-step mapping for authored-body undo and redo.
use std::fmt;

use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

use crate::source_custody::{
    WorthQueryAnyCorrectionCustodyOutcome, WorthQueryAnySourceCompletion,
    WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep,
    WorthQueryAnyTerminalSourceEvidence, WorthQuerySourceCustodyPending,
};

use super::{map_undo_denial, WorthQueryHomeUndoDenial};
use worth_query_publication::facade::application_entry::WorthQueryClosedOperation;

/// Outcome of an authored-body undo or redo via [`WorthQueryAnySourceCustody`].
#[must_use = "handle the committed correction or resume pending source custody"]
pub enum WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    Committed {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryAnySourceCompletion,
    },
    Declined {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryAnyTerminalSourceEvidence,
    },
    Pending {
        custody: WorthQueryAnySourceCustody<Schema, Program>,
        cause: WorthQuerySourceCustodyPending,
    },
    IdempotencyIntentDrift,
    PublicationPending,
    ProductUnpublished(ProductUnpublishedRecoveryHandle),
}

impl<Schema, Program> WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    /// Returns the live commit receipt when this correction committed freshly.
    pub fn live_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        match self {
            Self::Committed { evidence, .. } => evidence.live_receipt(),
            _ => None,
        }
    }
}

impl<Schema, Program> fmt::Debug for WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Committed { .. } => f.write_str("Committed(..)"),
            Self::Declined { evidence, .. } => f.debug_tuple("Declined").field(evidence).finish(),
            Self::Pending { cause, .. } => f.debug_tuple("Pending").field(cause).finish(),
            Self::IdempotencyIntentDrift => f.write_str("IdempotencyIntentDrift"),
            Self::PublicationPending => f.write_str("PublicationPending"),
            Self::ProductUnpublished(_) => f.write_str("ProductUnpublished(..)"),
        }
    }
}

pub(super) fn map_any_custody_outcome<Schema, Program>(
    outcome: WorthQueryAnyCorrectionCustodyOutcome<Schema, Program>,
) -> Result<WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program>, WorthQueryHomeUndoDenial> {
    use WorthQueryHomeAuthoredCorrectionOutcome as Out;
    match outcome {
        WorthQueryAnyCorrectionCustodyOutcome::Custody(step) => Ok(match step {
            WorthQueryAnySourceCustodyStep::Committed { closed, evidence } => {
                Out::Committed { closed, evidence }
            }
            WorthQueryAnySourceCustodyStep::Declined { closed, evidence } => {
                Out::Declined { closed, evidence }
            }
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                Out::Pending { custody, cause }
            }
        }),
        WorthQueryAnyCorrectionCustodyOutcome::PublicationRefused(denial) => {
            Err(map_undo_denial(denial))
        }
        WorthQueryAnyCorrectionCustodyOutcome::IdempotencyIntentDrift => {
            Ok(Out::IdempotencyIntentDrift)
        }
        WorthQueryAnyCorrectionCustodyOutcome::PublicationPending => Ok(Out::PublicationPending),
        WorthQueryAnyCorrectionCustodyOutcome::ProductUnpublished(handle) => {
            Ok(Out::ProductUnpublished(handle))
        }
    }
}
