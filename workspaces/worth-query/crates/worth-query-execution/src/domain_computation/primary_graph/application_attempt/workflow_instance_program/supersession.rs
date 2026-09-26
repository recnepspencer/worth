//! A start names the definition it read as current. When its branch holds
//! another definition current, or none, the start is refused before any
//! effect with an outcome naming what the branch holds instead. A retry of
//! a start already recorded still replays.

use super::super::{
    PublishedWorkflowDefinitionRef, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
};
use super::publication::WorkflowInstanceStartOutcome;
use crate::domain_computation::primary_graph::application_discovery::WorkflowDefinitionCurrentness;

/// A start that named a definition its branch has since superseded. Nothing
/// was written; `current()` names the definition to start from instead.
#[derive(Debug)]
pub struct SupersededWorkflowDefinitionStart {
    requested: PublishedWorkflowDefinitionRef,
    current: PublishedWorkflowDefinitionRef,
}

impl SupersededWorkflowDefinitionStart {
    pub const fn requested(&self) -> &PublishedWorkflowDefinitionRef {
        &self.requested
    }

    /// The definition the branch held current when the start was refused.
    /// It grants nothing: a start from it is checked like any other.
    pub const fn current(&self) -> &PublishedWorkflowDefinitionRef {
        &self.current
    }
}

/// A start that named a definition of a retired lineage. Nothing was
/// written, and no definition is current until one is published again.
#[derive(Debug)]
pub struct RetiredWorkflowDefinitionStart {
    requested: PublishedWorkflowDefinitionRef,
}

impl RetiredWorkflowDefinitionStart {
    pub const fn requested(&self) -> &PublishedWorkflowDefinitionRef {
        &self.requested
    }
}

/// Why a prepared start may not write a new instance.
pub(super) enum WorkflowStartSupersession {
    Superseded(SupersededWorkflowDefinitionStart),
    Retired(RetiredWorkflowDefinitionStart),
}

impl WorkflowStartSupersession {
    /// `None` when `requested` is what the branch holds current.
    pub(super) fn of(
        requested: &PublishedWorkflowDefinitionRef,
        currentness: WorkflowDefinitionCurrentness,
    ) -> Option<Self> {
        let requested = requested.clone();
        match currentness {
            WorkflowDefinitionCurrentness::Current(current)
                if current.entity_id() == requested.entity_id() =>
            {
                None
            }
            WorkflowDefinitionCurrentness::Current(current) => {
                Some(Self::Superseded(SupersededWorkflowDefinitionStart {
                    requested,
                    current,
                }))
            }
            WorkflowDefinitionCurrentness::Retired => {
                Some(Self::Retired(RetiredWorkflowDefinitionStart { requested }))
            }
        }
    }

    pub(super) const fn denial_kind(&self) -> WorthQueryApplicationAttemptDenialKind {
        match self {
            Self::Superseded(_) => {
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionSuperseded
            }
            Self::Retired(_) => WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionRetired,
        }
    }

    /// The typed outcome for the commit's own refusal of this start. Any
    /// other outcome, such as a replay, passes through unchanged.
    pub(super) fn outcome(
        self,
        outcome: WorthQueryApplicationCommitOutcome,
    ) -> Result<WorkflowInstanceStartOutcome, WorthQueryApplicationCommitOutcome> {
        let refused = WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
            kind: self.denial_kind(),
        };
        match outcome {
            WorthQueryApplicationCommitOutcome::Denied(denial) if denial.kind() == refused => {
                Ok(match self {
                    Self::Superseded(start) => WorkflowInstanceStartOutcome::Superseded(start),
                    Self::Retired(start) => WorkflowInstanceStartOutcome::Retired(start),
                })
            }
            other => Err(other),
        }
    }
}
