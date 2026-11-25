//! The typed cause of an occurrence whose operation catch-up could not fire.
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind as CommitDenial, WorthQueryConditionalExecutionCause,
    WorthQueryConditionalExecutionTerminal, WorthQueryConditionalReentryFailure as Reentry,
    WorthQueryTemporalInvocationFailureKind as Invocation,
};

/// What refused an occurrence's operation, classified from its typed cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalOccurrenceRefusalKind {
    /// A `unique` field the fire writes, an `on_fire` create included, holds
    /// that value on another entity, or the fire writes one value twice.
    UniqueValueTaken,
    /// The fire could not resolve a row it reads: its rule row, or the
    /// target an `on_fire ... in Target by key` hook acts in.
    ContextUnresolved,
    /// An effect the fire authored, an `on_fire` hook's included, was refused.
    EffectRefused,
    /// Commit validation or an installed invariant refused the fire.
    CommitRefused,
}

/// The first occurrence a catch-up call stopped on with
/// [`OperationRefused`](super::WorthQueryTemporalCatchUpStatus::OperationRefused):
/// how its processing ended and its typed execution cause. The occurrence
/// committed nothing and stays due.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalOccurrenceRefusal {
    terminal: WorthQueryConditionalExecutionTerminal,
    cause: Option<WorthQueryConditionalExecutionCause>,
}

impl WorthQueryTemporalOccurrenceRefusal {
    pub(super) fn new(
        terminal: WorthQueryConditionalExecutionTerminal,
        cause: Option<WorthQueryConditionalExecutionCause>,
    ) -> Self {
        Self { terminal, cause }
    }

    /// How the occurrence's processing ended.
    pub const fn terminal(&self) -> WorthQueryConditionalExecutionTerminal {
        self.terminal
    }

    /// The exact execution cause, when one was recorded.
    pub const fn cause(&self) -> Option<&WorthQueryConditionalExecutionCause> {
        self.cause.as_ref()
    }

    /// The closed classification of [`Self::cause`].
    pub fn kind(&self) -> WorthQueryTemporalOccurrenceRefusalKind {
        use WorthQueryConditionalExecutionCause as Cause;
        use WorthQueryTemporalOccurrenceRefusalKind as Kind;
        match &self.cause {
            Some(Cause::ApplicationCommitDenied(CommitDenial::UniqueValueTaken)) => {
                Kind::UniqueValueTaken
            }
            Some(Cause::Reentry(Reentry::Invocation(failure))) => match failure.kind() {
                Invocation::ProjectionRejected => Kind::ContextUnresolved,
                Invocation::InvocationRejected => Kind::EffectRefused,
            },
            _ => Kind::CommitRefused,
        }
    }
}
