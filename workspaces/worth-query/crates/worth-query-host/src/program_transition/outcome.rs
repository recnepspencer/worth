//! Typed outcomes and refusals for declarative Directory Home transitions.

use crate::application_home::{
    WorthQueryApplicationHome, WorthQueryHomePredecessorRefusal,
    WorthQueryHomeProgramTransitionOpenRefused,
};
use worth_query_declaration::facade::{
    application_schema::ApplicationSchemaComposition,
    program_transition::{
        WorthQueryProgramTransitionDeclarationDenial, WorthQueryProgramTransitionIndexRebuild,
    },
};

use super::config::WorthQueryProgramTransitionConfigDenial;

/// Descriptive outcome of opening a Directory Home through a declared transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryTransitionOpenOutcome {
    /// The Directory Home was already on the target program version; no
    /// transition or fresh initializer ran.
    OpenedCurrent { program_version: u32 },
    /// The Directory Home did not exist yet and was created directly on the
    /// target program version without invoking any predecessor or transition.
    CreatedFresh { program_version: u32 },
    /// The Directory Home was upgraded atomically in-place from `from_version`
    /// to `to_version`, retaining surviving records and backfilling declared defaults.
    UpgradedInPlace {
        from_version: u32,
        to_version: u32,
        backfilled_records: usize,
        indexes_marked_for_rebuild: Vec<WorthQueryProgramTransitionIndexRebuild>,
    },
}

/// Live opened Directory Home paired with its exact typed open/upgrade outcome.
pub struct WorthQueryTransitionOpenedHome<Schema, Program> {
    home: WorthQueryApplicationHome<Schema, Program>,
    outcome: WorthQueryTransitionOpenOutcome,
}

impl<Schema, Program> WorthQueryTransitionOpenedHome<Schema, Program> {
    pub(super) fn new(
        home: WorthQueryApplicationHome<Schema, Program>,
        outcome: WorthQueryTransitionOpenOutcome,
    ) -> Self {
        Self { home, outcome }
    }

    pub fn home(&self) -> &WorthQueryApplicationHome<Schema, Program> {
        &self.home
    }

    pub fn outcome(&self) -> &WorthQueryTransitionOpenOutcome {
        &self.outcome
    }

    pub fn into_home(self) -> WorthQueryApplicationHome<Schema, Program> {
        self.home
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationHome<Schema, Program>,
        WorthQueryTransitionOpenOutcome,
    ) {
        (self.home, self.outcome)
    }
}

/// Typed, fail-closed refusal for declarative Directory Home program transitions.
pub enum WorthQueryProgramTransitionOpenDenial<
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
> {
    /// Declared transition metadata failed validation (e.g. downgrade `v3 -> v2`,
    /// non-advancing version, identical program identity, or duplicate default).
    Declaration(WorthQueryProgramTransitionDeclarationDenial),
    /// Open configuration failed boundedness validation before touching disk.
    InvalidConfiguration(WorthQueryProgramTransitionConfigDenial),
    /// The Directory Home on disk does not match the transition's declared source program.
    PredecessorMismatch {
        kind: WorthQueryHomePredecessorRefusal,
        refused: WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    },
    /// A transition entity scan exceeded `max_entities_per_kind` before
    /// publishing any durable transition intent.
    EntityScanLimitExceeded {
        maximum_entities_per_kind: usize,
        refused: WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    },
    /// Execution budget, work ceiling, allocation limit, or cancellation stopped
    /// the transition without promoting the successor.
    BudgetOrCancellationStopped {
        refused: WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    },
    /// Any other underlying Host transition or opening refusal, preserving full custody.
    Host(WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>),
}

impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    WorthQueryProgramTransitionOpenDenial<A, AP, B, BP>
{
    pub(super) fn classify_host(
        refused: WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    ) -> Self {
        if let Some(kind) = refused.predecessor_mismatch() {
            return Self::PredecessorMismatch { kind, refused };
        }
        if let Some(maximum_entities_per_kind) = refused.entity_scan_limit_exceeded() {
            return Self::EntityScanLimitExceeded {
                maximum_entities_per_kind,
                refused,
            };
        }
        if refused.is_budget_or_cancellation_stop() {
            return Self::BudgetOrCancellationStopped { refused };
        }
        Self::Host(refused)
    }

    /// Borrow the underlying Host refusal when the rejection occurred after
    /// declaration and configuration checks passed.
    pub fn host_refusal(
        &self,
    ) -> Option<&WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>> {
        match self {
            Self::Declaration(_) | Self::InvalidConfiguration(_) => None,
            Self::PredecessorMismatch { refused, .. }
            | Self::EntityScanLimitExceeded { refused, .. }
            | Self::BudgetOrCancellationStopped { refused }
            | Self::Host(refused) => Some(refused),
        }
    }

    /// Consume this denial into the underlying Host refusal custody when present.
    pub fn into_host_refusal(
        self,
    ) -> Option<WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>> {
        match self {
            Self::Declaration(_) | Self::InvalidConfiguration(_) => None,
            Self::PredecessorMismatch { refused, .. }
            | Self::EntityScanLimitExceeded { refused, .. }
            | Self::BudgetOrCancellationStopped { refused }
            | Self::Host(refused) => Some(refused),
        }
    }
}

impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP> std::fmt::Debug
    for WorthQueryProgramTransitionOpenDenial<A, AP, B, BP>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declaration(cause) => f.debug_tuple("Declaration").field(cause).finish(),
            Self::InvalidConfiguration(cause) => {
                f.debug_tuple("InvalidConfiguration").field(cause).finish()
            }
            Self::PredecessorMismatch { kind, refused } => f
                .debug_struct("PredecessorMismatch")
                .field("kind", kind)
                .field("refused", refused)
                .finish(),
            Self::EntityScanLimitExceeded {
                maximum_entities_per_kind,
                refused,
            } => f
                .debug_struct("EntityScanLimitExceeded")
                .field("maximum_entities_per_kind", maximum_entities_per_kind)
                .field("refused", refused)
                .finish(),
            Self::BudgetOrCancellationStopped { refused } => f
                .debug_struct("BudgetOrCancellationStopped")
                .field("refused", refused)
                .finish(),
            Self::Host(refused) => f.debug_tuple("Host").field(refused).finish(),
        }
    }
}
